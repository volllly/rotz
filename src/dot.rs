mod repr {
  use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
  };

  use derive_more::IsVariant;
  #[cfg(test)]
  use fake::{Dummy, Fake};
  use serde::Deserialize;
  use somok::Somok;

  use crate::helpers::os;

  #[derive(Deserialize, Debug, Default)]
  #[cfg_attr(test, derive(Dummy))]
  struct DotSimplified {
    #[serde(flatten)]
    capabilities: Capabilities,
  }

  #[derive(Deserialize, Debug, Default, Clone)]
  #[cfg_attr(test, derive(Dummy))]
  pub struct Dot {
    pub global: Option<Box<Capabilities>>,
    pub windows: Option<Box<Capabilities>>,
    pub linux: Option<Box<Capabilities>>,
    pub darwin: Option<Box<Capabilities>>,
    #[serde(rename = "windows|linux", alias = "linux|windows")]
    pub windows_linux: Option<Box<Capabilities>>,
    #[serde(rename = "linux|darwin", alias = "darwin|linux")]
    pub linux_darwin: Option<Box<Capabilities>>,
    #[serde(rename = "darwin|windows", alias = "windows|darwin")]
    pub darwin_windows: Option<Box<Capabilities>>,
  }

  #[derive(Deserialize, Clone, Default, Debug)]
  #[cfg_attr(test, derive(Dummy))]
  pub struct Capabilities {
    pub(super) links: Option<Links>,
    pub(super) installs: Option<Installs>,
    #[serde(flatten)]
    pub(super) depends: Option<Depends>,
  }

  #[derive(Deserialize, Clone, Debug)]
  #[serde(untagged)]
  #[cfg_attr(test, derive(Dummy))]
  pub enum Links {
    One {
      #[serde(flatten)]
      links: HashMap<PathBuf, PathBuf>,
    },
    Many {
      #[serde(flatten)]
      links: HashMap<PathBuf, HashSet<PathBuf>>,
    },
  }

  #[derive(Deserialize, Clone, Debug, IsVariant)]
  #[serde(untagged)]
  #[cfg_attr(test, derive(Dummy))]
  pub enum Installs {
    None(bool),
    Simple(String),
    Full {
      cmd: String,
      #[serde(default)]
      depends: HashSet<String>,
    },
  }

  #[derive(Deserialize, Clone, Debug)]
  #[cfg_attr(test, derive(Dummy))]
  pub struct Depends {
    pub(super) depends: HashSet<String>,
  }

  #[cfg(feature = "toml")]
  pub type ParseError = serde_toml::de::Error;
  #[cfg(feature = "yaml")]
  pub type ParseError = serde_yaml::Error;
  #[cfg(feature = "json")]
  pub type ParseError = serde_json::Error;

  #[cfg(feature = "toml")]
  fn parse_inner<T: for<'de> Deserialize<'de>>(value: &str) -> Result<T, ParseError> {
    serde_toml::from_str(value)
  }

  #[cfg(feature = "yaml")]
  fn parse_inner<T: for<'de> Deserialize<'de> + Default>(value: &str) -> Result<T, ParseError> {
    match serde_yaml::from_str::<T>(value) {
      Ok(ok) => ok.okay(),
      Err(err) => match err.location() {
        Some(_) => err.error(),
        None => T::default().okay(),
      },
    }
  }

  #[cfg(feature = "json")]
  fn parse_inner<T: for<'de> Deserialize<'de>>(value: &str) -> Result<T, ParseError> {
    serde_json::from_str(value)
  }
  impl Dot {
    pub(crate) fn parse(value: &str) -> Result<Self, ParseError> {
      let parsed = parse_inner::<DotSimplified>(value)?;

      if let DotSimplified {
        capabilities: Capabilities {
          links: None,
          installs: None,
          depends: None,
        },
      } = &parsed
      {
        parse_inner::<Self>(value)
      } else {
        Self {
          global: parsed.capabilities.boxed().some(),
          ..Default::default()
        }
        .okay()
      }
    }
  }

  impl From<Dot> for Capabilities {
    fn from(
      Dot {
        global,
        windows,
        linux,
        darwin,
        windows_linux,
        linux_darwin,
        darwin_windows,
      }: Dot,
    ) -> Self {
      let mut capabilities: Option<Capabilities> = global.map(|g| (*g).clone());

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

  impl Merge<Option<Box<Capabilities>>> for Option<Capabilities> {
    fn merge(self, merge: Option<Box<Capabilities>>) -> Self {
      if let Some(s) = self {
        if let Some(merge) = merge { s.merge(*merge) } else { s }.some()
      } else {
        merge.map(|g| *g)
      }
    }
  }

  impl Merge<Self> for Capabilities {
    fn merge(mut self, Capabilities { mut links, installs, depends }: Self) -> Self {
      if let Some(self_links) = &mut self.links {
        if let Links::One { links: self_links_one } = self_links {
          *self_links = Links::Many {
            links: self_links_one
              .iter_mut()
              .map(|l| {
                let mut hs = HashSet::new();
                hs.insert(l.1.clone());
                (l.0.clone(), hs)
              })
              .collect(),
          };
        }
      }

      if let Some(match_links) = &mut links {
        if let Links::One { links: match_links_one } = match_links {
          *match_links = Links::Many {
            links: match_links_one
              .iter_mut()
              .map(|l| {
                let mut hs = HashSet::new();
                hs.insert(l.1.clone());
                (l.0.clone(), hs)
              })
              .collect(),
          };
        }
      }
      if let Some(self_links) = &mut self.links {
        if let Some(merge_links) = &mut links {
          if let Links::Many { links: self_links_many } = self_links {
            if let Links::Many { links: merge_links_many } = merge_links {
              for l in merge_links_many.iter_mut() {
                if self_links_many.contains_key(l.0) {
                  let self_links_many_value = self_links_many.get_mut(l.0).unwrap();
                  self_links_many_value.extend(l.1.clone());
                } else {
                  self_links_many.insert(l.0.clone(), l.1.clone());
                }
              }
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
            let mut depends_outer: HashSet<String> = HashSet::new();

            match installs {
              Installs::Simple(cmd) => cmd_outer = cmd,
              Installs::Full { cmd, depends } => {
                cmd_outer = cmd;
                depends_outer = depends;
              }
              Installs::None(_) => panic!(),
            }

            *i = match i {
              Installs::None(_) => Installs::Full {
                cmd: cmd_outer,
                depends: depends_outer,
              },
              Installs::Simple(_) => Installs::Full {
                cmd: cmd_outer,
                depends: depends_outer,
              },
              Installs::Full { cmd: _, depends } => {
                depends_outer.extend(depends.clone());
                Installs::Full {
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
          d.depends.extend(depends.depends);
        }
      } else {
        self.depends = depends;
      }

      self
    }
  }
}

use std::{
  collections::{HashMap, HashSet},
  fs,
  path::{Path, PathBuf},
  str::FromStr,
};

use crossterm::style::Stylize;
use itertools::Itertools;
use miette::Diagnostic;
pub use repr::Merge;
use somok::Somok;

use self::repr::Capabilities;
use crate::{helpers::os, FILE_EXTENSION};

#[derive(Clone, Debug)]
pub struct Installs {
  pub(crate) cmd: String,
  pub(crate) depends: HashSet<String>,
}

impl From<repr::Installs> for Option<Installs> {
  fn from(from: repr::Installs) -> Self {
    match from {
      repr::Installs::None(_) => None,
      repr::Installs::Simple(cmd) => Installs { cmd, depends: Default::default() }.some(),
      repr::Installs::Full { cmd, depends } => Installs { cmd, depends }.some(),
    }
  }
}

#[derive(Default, Clone, Debug)]
pub struct Dot {
  pub(crate) links: Option<HashMap<PathBuf, HashSet<PathBuf>>>,
  pub(crate) installs: Option<Installs>,
  pub(crate) depends: Option<HashSet<String>>,
}

impl Merge<&Self> for Dot {
  fn merge(mut self, merge: &Self) -> Self {
    if let Some(links) = &merge.links {
      if let Some(l) = &mut self.links {
        l.extend(links.clone());
      } else {
        self.links = links.clone().some();
      }
    }

    if let Some(installs) = &merge.installs {
      if let Some(i) = &mut self.installs {
        i.cmd = installs.cmd.clone();
        i.depends.extend(installs.depends.clone());
      } else {
        self.installs = installs.clone().some();
      }
    }

    if let Some(depends) = &merge.depends {
      if let Some(d) = &mut self.depends {
        d.extend(depends.clone());
      } else {
        self.depends = depends.clone().some();
      }
    }

    self
  }
}

fn from_str_with_defaults(s: &str, defaults: Option<&Capabilities>) -> Result<Dot, repr::ParseError> {
  let repr::Dot {
    global,
    windows,
    linux,
    darwin,
    windows_linux,
    linux_darwin,
    darwin_windows,
  } = repr::Dot::parse(s)?;

  let capabilities: Option<Capabilities> = if let Some(defaults) = defaults { (*defaults).clone().some() } else { None };

  let mut capabilities: Option<Capabilities> = global.map_or(capabilities.clone(), |g| capabilities.merge(g.some()));

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

  if let Some(capabilities) = capabilities {
    Dot {
      links: capabilities.links.map(|c| match c {
        repr::Links::One { links } => links
          .into_iter()
          .map(|l| {
            let mut hs = HashSet::<PathBuf>::new();
            hs.insert(l.1);
            (l.0, hs)
          })
          .collect(),
        repr::Links::Many { links } => links,
      }),
      installs: capabilities.installs.and_then(|i| i.into()),
      depends: capabilities.depends.map(|c| c.depends),
    }
  } else {
    Dot::default()
  }
  .okay()
}

impl FromStr for Dot {
  type Err = repr::ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    from_str_with_defaults(s, None)
  }
}

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum Error {
  #[error("Could not find dot directory \"{0}\"")]
  #[diagnostic(code(dot::filename::get), help("Did you enter a valid file?"))]
  PathFind(PathBuf),

  #[error("Could not parse dot directory \"{0}\"")]
  #[diagnostic(code(dotfiles::filename::parse), help("Did you enter a valid file?"))]
  PathParse(PathBuf),

  #[error("Could not read dotfiles directory \"{0}\"")]
  #[diagnostic(code(dotfiles::directory::read), help("did you change/set the dotfiles path?"))]
  DotfileDir(PathBuf, #[source] std::io::Error),

  #[error("Could not read dot file")]
  #[diagnostic(code(dot::read))]
  ReadingDot(#[source] std::io::Error),

  #[cfg(feature = "yaml")]
  #[error("Could not parse dot \"{0}\"")]
  #[diagnostic(code(dot::parse))]
  ParseDot(PathBuf, #[source] serde_yaml::Error),

  #[error("Io Error on file \"{0}\"")]
  #[diagnostic(code(io::generic))]
  Io(PathBuf, #[source] std::io::Error),
}

pub fn read_dots(dotfiles_path: &Path, dots: &[String]) -> miette::Result<Vec<(String, Dot)>> {
  let defaults = dotfiles_path.join(format!("dots.{FILE_EXTENSION}"));
  let defaults = match fs::read_to_string(defaults.clone()) {
    Ok(text) => repr::Dot::parse(&text).map_err(|e| Error::ParseDot(defaults, e))?.some(),
    Err(err) => match err.kind() {
      std::io::ErrorKind::NotFound => None,
      _ => panic!("{}", err),
    },
  };
  let defaults = defaults.map(Into::<Capabilities>::into);

  let wildcard = dots.contains(&"*".to_string());

  let paths = fs::read_dir(&dotfiles_path)
    .map_err(|e| Error::DotfileDir(dotfiles_path.to_path_buf(), e))?
    .map(|d| match d {
      Ok(d) => d.path().okay(),
      Err(err) => Error::ReadingDot(err).error(),
    })
    .filter_ok(|p| p.is_dir());

  let dotfiles = crate::helpers::join_err_result(paths.collect())?
    .into_iter()
    .map(|p| {
      let name = p
        .file_name()
        .ok_or_else(|| Error::PathFind(p.clone()))?
        .to_str()
        .ok_or_else(|| Error::PathParse(p.clone()))?
        .to_string();
      Ok::<(String, PathBuf), Error>((name, p))
    })
    .filter_ok(|p| wildcard || dots.contains(&p.0))
    .map_ok(|p| {
      (
        p.0,
        fs::read_to_string(p.1.join(format!("dot.{FILE_EXTENSION}"))).map_err(|e| Error::Io(p.1.join(format!("dot.{FILE_EXTENSION}")), e)),
      )
    });

  let dots = dotfiles.filter_map(|f| match f {
    Ok((name, Ok(text))) => match from_str_with_defaults(&text, defaults.as_ref()) {
      Ok(dot) => (name, dot).okay().some(),
      Err(err) => Error::ParseDot(Path::new(&format!("{name}/dot.{FILE_EXTENSION}")).to_path_buf(), err).error().some(),
    },
    Ok((_, Err(Error::Io(file, err)))) => match err.kind() {
      std::io::ErrorKind::NotFound => None,
      _ => Error::Io(file, err).error().some(),
    },
    Ok((_, Err(err))) | Err(err) => err.error().some(),
  });

  let dots = crate::helpers::join_err_result(dots.collect())?;
  if dots.is_empty() {
    println!("Warning: {}", "No dots found".yellow());
    return vec![].okay();
  }

  dots.okay()
}
