mod repr {
  use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
  };

  use derive_more::IsVariant;
  #[cfg(test)]
  use fake::{Dummy, Fake};
  use serde::Deserialize;
  use tap::{Conv, Pipe};
  use velcro::hash_set;

  use crate::{
    helpers::{self, os},
    FileFormat,
  };

  #[derive(Deserialize, Debug, Default)]
  #[cfg_attr(test, derive(Dummy))]
  #[serde(deny_unknown_fields)]
  struct DotSimplified {
    pub(super) links: Option<Links>,
    pub(super) installs: Option<Installs>,
    pub(super) depends: Option<HashSet<String>>,
  }

  #[derive(Deserialize, Debug, Default, Clone)]
  #[cfg_attr(test, derive(Dummy))]
  #[serde(deny_unknown_fields)]
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
  #[serde(deny_unknown_fields)]
  pub struct Capabilities {
    pub(super) links: Option<Links>,
    pub(super) installs: Option<Installs>,
    pub(super) depends: Option<HashSet<String>>,
  }

  impl From<DotSimplified> for Capabilities {
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
  pub enum Links {
    One(HashMap<PathBuf, PathBuf>),
    Many(HashMap<PathBuf, HashSet<PathBuf>>),
  }

  #[derive(Deserialize, Clone, Debug, IsVariant)]
  #[serde(untagged)]
  #[cfg_attr(test, derive(Dummy))]
  #[serde(deny_unknown_fields)]
  pub enum Installs {
    None(bool),
    Simple(String),
    Full {
      cmd: String,
      #[serde(default)]
      depends: HashSet<String>,
    },
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

  impl Dot {
    pub(crate) fn parse(value: &str, format: FileFormat) -> Result<Self, Vec<helpers::ParseError>> {
      match parse_inner::<Self>(value, format) {
        Ok(parsed) => parsed.pipe(Ok),
        Err(err) => Self {
          global: parse_inner::<DotSimplified>(value, format).map_err(|e| vec![err, e])?.conv::<Capabilities>().conv::<Box<_>>().into(),
          ..Default::default()
        }
        .pipe(Ok),
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
      let mut capabilities: Option<Self> = global.map(|g| (*g).clone());

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
        if let Some(merge) = merge { s.merge(*merge) } else { s }.into()
      } else {
        merge.map(|g| *g)
      }
    }
  }

  impl Merge<Self> for Capabilities {
    fn merge(mut self, Self { mut links, installs, depends }: Self) -> Self {
      if let Some(self_links) = &mut self.links {
        if let Links::One(self_links_one) = self_links {
          *self_links = Links::Many(self_links_one.iter_mut().map(|l| (l.0.clone(), hash_set!(l.1.clone()))).collect());
        }
      }

      if let Some(match_links) = &mut links {
        if let Links::One(match_links_one) = match_links {
          *match_links = Links::Many(match_links_one.iter_mut().map(|l| (l.0.clone(), hash_set!(l.1.clone()))).collect());
        }
      }
      if let Some(self_links) = &mut self.links {
        if let Some(merge_links) = &mut links {
          if let Links::Many(self_links_many) = self_links {
            if let Links::Many(merge_links_many) = merge_links {
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
            let mut depends_outer = HashSet::<String>::new();

            match installs {
              Installs::Simple(cmd) => cmd_outer = cmd,
              Installs::Full { cmd, depends } => {
                cmd_outer = cmd;
                depends_outer = depends;
              }
              Installs::None(_) => panic!(),
            }

            *i = match i {
              Installs::None(_) | Installs::Simple(_) => Installs::Full {
                cmd: cmd_outer,
                depends: depends_outer,
              },
              Installs::Full { depends, .. } => {
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
          d.extend(depends);
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
};

use crossterm::style::Stylize;
use itertools::Itertools;
use miette::{Diagnostic, NamedSource, Report, SourceSpan};
use path_slash::PathBufExt;
use repr::Merge;
use tap::{Pipe, TryConv};
use velcro::hash_set;
use walkdir::WalkDir;
use wax::Pattern;

use self::repr::Capabilities;
use crate::{
  config::Config,
  helpers::{self, os},
  templating::{self, Parameters},
  FileFormat, FILE_EXTENSIONS_GLOB,
};

#[derive(Clone, Debug)]
pub struct Installs {
  pub(crate) cmd: String,
  pub(crate) depends: HashSet<String>,
}

impl From<repr::Installs> for Option<Installs> {
  fn from(from: repr::Installs) -> Self {
    match from {
      repr::Installs::None(_) => None,
      repr::Installs::Simple(cmd) => Installs { cmd, depends: Default::default() }.into(),
      repr::Installs::Full { cmd, depends } => Installs { cmd, depends }.into(),
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
        self.links = links.clone().into();
      }
    }

    if let Some(installs) = &merge.installs {
      if let Some(i) = &mut self.installs {
        i.cmd = installs.cmd.clone();
        i.depends.extend(installs.depends.clone());
      } else {
        self.installs = installs.clone().into();
      }
    }

    if let Some(depends) = &merge.depends {
      if let Some(d) = &mut self.depends {
        d.extend(depends.clone());
      } else {
        self.depends = depends.clone().into();
      }
    }

    self
  }
}

fn from_str_with_defaults(s: &str, format: FileFormat, defaults: Option<&Capabilities>) -> Result<Dot, Vec<helpers::ParseError>> {
  let repr::Dot {
    global,
    windows,
    linux,
    darwin,
    windows_linux,
    linux_darwin,
    darwin_windows,
  } = repr::Dot::parse(s, format)?;

  let capabilities: Option<Capabilities> = defaults.and_then(|defaults| (*defaults).clone().into());

  let mut capabilities: Option<Capabilities> = global.map_or(capabilities.clone(), |g| capabilities.merge(g.into()));

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
        repr::Links::One(links) => links.into_iter().map(|l| (l.0, hash_set!(l.1))).collect(),
        repr::Links::Many(links) => links,
      }),
      installs: capabilities.installs.and_then(Into::into),
      depends: capabilities.depends,
    }
  } else {
    Dot::default()
  }
  .pipe(Ok)
}

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum Error {
  #[error("Could not get relative dot directory")]
  #[diagnostic(code(dotfiles::filename::strip))]
  PathStrip(#[source] std::path::StripPrefixError),

  #[error("Could not read dot file")]
  #[diagnostic(code(dot::read))]
  ReadingDot(#[source] std::io::Error),

  #[error("Error walking dotfiles")]
  #[diagnostic(code(dotfiles::walk))]
  WalkingDotfiles(#[source] walkdir::Error),

  #[error("The \"dots.{0}\" file is deprecated and support will be removed in a future version.")]
  #[diagnostic(code(dot::defaults::deprecated), severity(warning), help("Please rename this file to \"defaults.{0}\"."))]
  DotsDeprecated(String, #[label] SourceSpan, #[source_code] String),

  #[cfg(feature = "yaml")]
  #[error("Could not parse dot")]
  #[diagnostic(code(dot::parse))]
  ParseDot(#[source_code] NamedSource, #[label] SourceSpan, #[related] Vec<helpers::ParseError>),

  #[cfg(feature = "yaml")]
  #[error("Could not render template for dot")]
  #[diagnostic(code(dot::render))]
  RenderDot(#[source_code] NamedSource, #[label] SourceSpan, #[source] templating::Error),

  #[error("Io Error on file \"{0}\"")]
  #[diagnostic(code(io::generic))]
  Io(PathBuf, #[source] std::io::Error),

  #[error("Could not parse dependency path \"{0}\"")]
  #[diagnostic(code(glob::parse))]
  ParseDependency(PathBuf, #[source] std::io::Error),

  #[error("Could not parse dot name \"{0}\"")]
  #[diagnostic(code(glob::parse))]
  ParseName(String, #[source] std::io::Error),

  #[error(transparent)]
  MultipleErrors(#[from] helpers::MultipleErrors),
}

pub fn read_dots(dotfiles_path: &Path, dots: &[String], config: &Config) -> miette::Result<Vec<(String, Dot)>> {
  let defaults = get_defaults(dotfiles_path)?;

  let dots = helpers::glob_from_vec(dots, &format!("/dot.{FILE_EXTENSIONS_GLOB}"))?;

  let paths = WalkDir::new(&dotfiles_path)
    .into_iter()
    .filter_ok(|e| !e.file_type().is_dir())
    .map(|d| -> Result<(std::string::String, std::path::PathBuf), Error> {
      let d = d.map_err(Error::WalkingDotfiles)?;
      let path = d.path().strip_prefix(dotfiles_path).map(Path::to_path_buf).map_err(Error::PathStrip)?;
      let absolutized = helpers::absolutize_virtually(&path).map_err(|e| Error::ParseName(path.to_string_lossy().to_string(), e))?;
      Ok((absolutized, path))
    })
    .filter_ok(|e| dots.is_match(e.0.as_str()))
    .map_ok(|e| {
      let format = e.1.as_path().try_conv::<FileFormat>().unwrap();
      (e.1, format)
    });

  let dotfiles = crate::helpers::join_err_result(paths.collect())?
    .into_iter()
    .map(|p| {
      let name = p.0.parent().unwrap().to_path_buf().to_slash_lossy().to_string();
      Ok::<(String, (PathBuf, FileFormat)), Error>((name, p))
    })
    .map_ok(|p| {
      (
        p.0,
        fs::read_to_string(dotfiles_path.join(&p.1 .0))
          .map(|d| (d, p.1 .1))
          .map_err(|e| Error::Io(dotfiles_path.join(p.1 .0), e)),
      )
    });

  let dots = dotfiles.filter_map(|f| match f {
    Ok((name, Ok((text, format)))) => {
      let parameters = Parameters { config, name: &name };
      let text = match templating::render(&text, &parameters) {
        Ok(text) => text,
        Err(err) => {
          return Error::RenderDot(NamedSource::new(format!("{name}/dot.{format}"), text.clone()), (0, text.len()).into(), err)
            .pipe(Err)
            .into()
        }
      };

      let defaults = if let Some((defaults, format)) = defaults.as_ref() {
        match templating::render(defaults, &parameters) {
          Ok(rendered) => match repr::Dot::parse(&rendered, *format) {
            Ok(parsed) => Into::<Capabilities>::into(parsed).into(),
            Err(err) => return Error::ParseDot(NamedSource::new(defaults, defaults.to_string()), (0, defaults.len()).into(), err).pipe(Err).into(),
          },
          Err(err) => {
            return Error::RenderDot(NamedSource::new(format!("{name}/dot.{format}"), defaults.to_string()), (0, defaults.len()).into(), err)
              .pipe(Err)
              .into()
          }
        }
      } else {
        None
      };

      match from_str_with_defaults(&text, format, defaults.as_ref()) {
        Ok(dot) => (name.clone(), dot).pipe(Ok).into(),
        Err(err) => Error::ParseDot(NamedSource::new(format!("{name}/dot.{format}"), text.clone()), (0, text.len()).into(), err)
          .pipe(Err)
          .into(),
      }
    }
    Ok((_, Err(Error::Io(file, err)))) => match err.kind() {
      std::io::ErrorKind::NotFound => None,
      _ => Error::Io(file, err).pipe(Err).into(),
    },
    Ok((_, Err(err))) | Err(err) => err.pipe(Err).into(),
  });

  let dots = canonicalize_dots(crate::helpers::join_err_result(dots.collect())?)?;

  if dots.is_empty() {
    println!("Warning: {}", "No dots found".yellow());
    return vec![].pipe(Ok);
  }

  dots.pipe(Ok)
}

fn get_defaults(dotfiles_path: &Path) -> Result<Option<(String, FileFormat)>, Error> {
  let mut defaults = helpers::get_file_with_format(dotfiles_path, "dots");
  if let Some(defaults) = &defaults {
    let path = defaults.0.to_string_lossy().to_string();
    println!(
      "Warning: {:?}",
      Report::new(Error::DotsDeprecated(defaults.1.to_string(), (path.rfind("dots").unwrap(), "dots".len()).into(), path))
    );
  } else {
    defaults = helpers::get_file_with_format(dotfiles_path, "dots");
  }

  if let Some(defaults) = defaults {
    match fs::read_to_string(defaults.0) {
      Ok(text) => (text, defaults.1).into(),
      Err(err) => match err.kind() {
        std::io::ErrorKind::NotFound => None,
        _ => Error::ReadingDot(err).pipe(Err)?,
      },
    }
  } else {
    None
  }
  .pipe(Ok)
}

fn canonicalize_dots(dots: Vec<(String, Dot)>) -> Result<Vec<(String, Dot)>, helpers::MultipleErrors> {
  let dots = dots.into_iter().map(|mut dot| {
    let name = helpers::absolutize_virtually(Path::new(&dot.0)).map_err(|e| Error::ParseName(dot.0.clone(), e))?;

    if let Some(installs) = &mut dot.1.installs {
      let depends = installs.depends.iter().map(|dependency| {
        let dependency_base = Path::new(&name).parent().unwrap_or_else(|| Path::new("")).join(dependency);

        let dependency_base = helpers::absolutize_virtually(&dependency_base).map_err(|e| Error::ParseDependency(dependency_base, e))?;
        dependency_base.pipe(Ok::<_, Error>)
      });
      installs.depends = helpers::join_err_result(depends.collect_vec())?.into_iter().collect::<HashSet<_>>();
    }

    if let Some(depends) = &dot.1.depends {
      let depends_mapped = depends.iter().map(|dependency| {
        let dependency_base = Path::new(&name).parent().unwrap_or_else(|| Path::new("")).join(dependency);

        let dependency_base = helpers::absolutize_virtually(&dependency_base).map_err(|e| Error::ParseDependency(dependency_base, e))?;
        dependency_base.pipe(Ok::<_, Error>)
      });
      dot.1.depends = Some(helpers::join_err_result(depends_mapped.collect_vec())?.into_iter().collect::<HashSet<_>>());
    }

    (name, dot.1).pipe(Ok::<_, Error>)
  });

  helpers::join_err_result(dots.collect_vec())
}

#[cfg(test)]
mod test;
