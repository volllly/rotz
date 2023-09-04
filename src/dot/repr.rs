use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
};

use derive_more::IsVariant;
#[cfg(test)]
use fake::Dummy;
use serde::Deserialize;
use tap::{Conv, Pipe};
#[cfg(feature = "profiling")]
use tracing::instrument;
use velcro::hash_set;

use crate::{
  helpers::{self, os},
  FileFormat,
};

#[derive(Deserialize, Debug, Default)]
#[cfg_attr(test, derive(Dummy))]
#[serde(deny_unknown_fields)]
struct DotSimplified {
  pub(super) links: Option<HashMap<PathBuf, LinksComplex>>,
  pub(super) installs: Option<InstallsComplex>,
  pub(super) depends: Option<HashSet<String>>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[cfg_attr(test, derive(Dummy))]
#[serde(deny_unknown_fields)]
struct DotComplex {
  pub global: Option<CapabilitiesComplex>,
  pub windows: Option<CapabilitiesComplex>,
  pub linux: Option<CapabilitiesComplex>,
  pub darwin: Option<CapabilitiesComplex>,
  #[serde(rename = "windows|linux", alias = "linux|windows")]
  pub windows_linux: Option<CapabilitiesComplex>,
  #[serde(rename = "linux|darwin", alias = "darwin|linux")]
  pub linux_darwin: Option<CapabilitiesComplex>,
  #[serde(rename = "darwin|windows", alias = "windows|darwin")]
  pub darwin_windows: Option<CapabilitiesComplex>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[cfg_attr(test, derive(Dummy))]
#[serde(deny_unknown_fields)]
pub struct DotCanonical {
  pub global: Option<CapabilitiesCanonical>,
  pub windows: Option<CapabilitiesCanonical>,
  pub linux: Option<CapabilitiesCanonical>,
  pub darwin: Option<CapabilitiesCanonical>,
  #[serde(rename = "windows|linux", alias = "linux|windows")]
  pub windows_linux: Option<CapabilitiesCanonical>,
  #[serde(rename = "linux|darwin", alias = "darwin|linux")]
  pub linux_darwin: Option<CapabilitiesCanonical>,
  #[serde(rename = "darwin|windows", alias = "windows|darwin")]
  pub darwin_windows: Option<CapabilitiesCanonical>,
}

impl From<DotComplex> for DotCanonical {
  #[cfg_attr(feature = "profiling", instrument)]
  fn from(value: DotComplex) -> Self {
    Self {
      global: value.global.map(Into::into),
      windows: value.windows.map(Into::into),
      linux: value.linux.map(Into::into),
      darwin: value.darwin.map(Into::into),
      windows_linux: value.windows_linux.map(Into::into),
      linux_darwin: value.linux_darwin.map(Into::into),
      darwin_windows: value.darwin_windows.map(Into::into),
    }
  }
}

#[derive(Deserialize, Clone, Default, Debug)]
#[cfg_attr(test, derive(Dummy))]
#[serde(deny_unknown_fields)]
struct CapabilitiesComplex {
  pub(super) links: Option<HashMap<PathBuf, LinksComplex>>,
  pub(super) installs: Option<InstallsComplex>,
  pub(super) depends: Option<HashSet<String>>,
}

#[derive(Deserialize, Clone, Default, Debug)]
#[cfg_attr(test, derive(Dummy))]
#[serde(deny_unknown_fields)]
pub struct CapabilitiesCanonical {
  pub(super) links: Option<HashMap<PathBuf, HashSet<PathBuf>>>,
  pub(super) installs: Option<InstallsCanonical>,
  pub(super) depends: Option<HashSet<String>>,
}

impl From<CapabilitiesComplex> for CapabilitiesCanonical {
  #[cfg_attr(feature = "profiling", instrument)]
  fn from(value: CapabilitiesComplex) -> Self {
    Self {
      links: value.links.map(|links| {
        links
          .into_iter()
          .map(|l| {
            (
              l.0,
              match l.1 {
                LinksComplex::One(o) => hash_set!(o),
                LinksComplex::Many(m) => m,
              },
            )
          })
          .collect::<HashMap<_, _>>()
      }),
      installs: value.installs.map(Into::into),
      depends: value.depends,
    }
  }
}

impl From<DotSimplified> for CapabilitiesComplex {
  fn from(from: DotSimplified) -> Self {
    Self {
      depends: from.depends,
      installs: from.installs,
      links: from.links,
    }
  }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
#[cfg_attr(test, derive(Dummy))]
#[serde(deny_unknown_fields)]
enum LinksComplex {
  One(PathBuf),
  Many(HashSet<PathBuf>),
}

#[derive(Deserialize, Clone, Debug, IsVariant)]
#[serde(untagged)]
#[cfg_attr(test, derive(Dummy))]
#[serde(deny_unknown_fields)]
enum InstallsComplex {
  None(bool),
  Simple(String),
  Full {
    cmd: String,
    #[serde(default)]
    depends: HashSet<String>,
  },
}

#[derive(Deserialize, Clone, Debug, IsVariant)]
#[serde(untagged)]
#[cfg_attr(test, derive(Dummy))]
#[serde(deny_unknown_fields)]
pub enum InstallsCanonical {
  None(bool),
  Full {
    cmd: String,
    #[serde(default)]
    depends: HashSet<String>,
  },
}

impl From<InstallsComplex> for InstallsCanonical {
  fn from(value: InstallsComplex) -> Self {
    match value {
      InstallsComplex::None(t) => InstallsCanonical::None(t),
      InstallsComplex::Simple(cmd) => InstallsCanonical::Full { cmd, depends: hash_set!() },
      InstallsComplex::Full { cmd, depends } => InstallsCanonical::Full { cmd, depends },
    }
  }
}

#[cfg(feature = "toml")]
fn parse_inner_toml<T: for<'de> Deserialize<'de>>(value: &str) -> Result<T, helpers::ParseError> {
  serde_toml::from_str::<T>(value)?.pipe(Ok)
}

#[cfg(feature = "yaml")]
fn parse_inner_yaml<T: for<'de> Deserialize<'de> + Default>(value: &str) -> Result<T, helpers::ParseError> {
  match serde_yaml::from_str::<T>(value) {
    Ok(ok) => ok.pipe(Ok),
    Err(err) => match err.location() {
      Some(_) => err.pipe(Err)?,
      None => T::default().pipe(Ok),
    },
  }
}

#[cfg(feature = "json")]
fn parse_inner_json<T: for<'de> Deserialize<'de>>(value: &str) -> Result<T, helpers::ParseError> {
  serde_json::from_str::<T>(value)?.pipe(Ok)
}

#[cfg_attr(feature = "profiling", instrument)]
fn parse_inner<T: for<'de> Deserialize<'de> + Default>(value: &str, format: FileFormat) -> Result<T, helpers::ParseError> {
  match format {
    #[cfg(feature = "yaml")]
    FileFormat::Yaml => parse_inner_yaml::<T>(value),
    #[cfg(feature = "toml")]
    FileFormat::Toml => parse_inner_toml::<T>(value),
    #[cfg(feature = "json")]
    FileFormat::Json => parse_inner_json::<T>(value),
  }
}

impl DotComplex {
  #[cfg_attr(feature = "profiling", instrument)]
  fn parse(value: &str, format: FileFormat) -> Result<Self, Vec<helpers::ParseError>> {
    match parse_inner::<Self>(value, format) {
      Ok(parsed) => parsed.pipe(Ok),
      Err(err) => Self {
        global: parse_inner::<DotSimplified>(value, format).map_err(|e| vec![err, e])?.conv::<CapabilitiesComplex>().into(),
        ..Default::default()
      }
      .pipe(Ok),
    }
  }
}

impl DotCanonical {
  #[cfg_attr(feature = "profiling", instrument)]
  pub(crate) fn parse(value: &str, format: FileFormat) -> Result<Self, Vec<helpers::ParseError>> {
    DotComplex::parse(value, format).map(Into::into)
  }
}

impl From<DotCanonical> for CapabilitiesCanonical {
  #[cfg_attr(feature = "profiling", instrument)]
  fn from(
    DotCanonical {
      global,
      windows,
      linux,
      darwin,
      windows_linux,
      linux_darwin,
      darwin_windows,
    }: DotCanonical,
  ) -> Self {
    let mut capabilities: Option<Self> = global;

    if os::OS.is_windows() {
      capabilities = capabilities.merge(windows_linux);
      capabilities = capabilities.merge(darwin_windows);
      capabilities = capabilities.merge(windows);
    } else if os::OS.is_linux() {
      capabilities = capabilities.merge(windows_linux);
      capabilities = capabilities.merge(linux_darwin);
      capabilities = capabilities.merge(linux);
    } else if os::OS.is_darwin() {
      capabilities = capabilities.merge(linux_darwin);
      capabilities = capabilities.merge(darwin_windows);
      capabilities = capabilities.merge(darwin);
    }

    capabilities.unwrap_or_default()
  }
}

pub trait Merge<T> {
  fn merge(self, merge: T) -> Self;
}

impl Merge<Option<CapabilitiesCanonical>> for Option<CapabilitiesCanonical> {
  #[cfg_attr(feature = "profiling", instrument)]
  fn merge(self, merge: Option<CapabilitiesCanonical>) -> Self {
    if let Some(s) = self {
      if let Some(merge) = merge { s.merge(merge) } else { s }.into()
    } else {
      merge
    }
  }
}

impl Merge<Self> for CapabilitiesCanonical {
  #[cfg_attr(feature = "profiling", instrument)]
  fn merge(mut self, Self { mut links, installs, depends }: Self) -> Self {
    if let Some(self_links) = &mut self.links {
      if let Some(merge_links) = &mut links {
        for l in &mut *merge_links {
          if self_links.contains_key(l.0) {
            let self_links_value = self_links.get_mut(l.0).unwrap();
            self_links_value.extend(l.1.clone());
          } else {
            self_links.insert(l.0.clone(), l.1.clone());
          }
        }
      }
    } else {
      self.links = links;
    }

    if let Some(i) = &mut self.installs {
      if let Some(installs) = installs {
        if installs.is_none() {
          self.installs = None;
        } else {
          let cmd_outer: String;
          let mut depends_outer;

          match installs {
            InstallsCanonical::Full { cmd, depends } => {
              cmd_outer = cmd;
              depends_outer = depends;
            }
            InstallsCanonical::None(_) => panic!(),
          }

          *i = match i {
            InstallsCanonical::None(_) => InstallsCanonical::Full {
              cmd: cmd_outer,
              depends: depends_outer,
            },
            InstallsCanonical::Full { depends, .. } => {
              depends_outer.extend(depends.clone());
              InstallsCanonical::Full {
                cmd: cmd_outer,
                depends: depends_outer,
              }
            }
          };
        }
      }
    } else {
      self.installs = installs;
    }

    if let Some(d) = &mut self.depends {
      if let Some(depends) = depends {
        d.extend(depends);
      }
    } else {
      self.depends = depends;
    }

    self
  }
}
