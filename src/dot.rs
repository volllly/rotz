mod repr {
  use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
  };

  #[cfg(test)]
  use fake::{Dummy, Fake};
  use serde::Deserialize;
  use somok::Somok;

  #[derive(Deserialize, Debug, Default)]
  #[cfg_attr(test, derive(Dummy))]
  struct DotSimplified {
    #[serde(flatten)]
    capabilities: Capabilities,
  }

  #[derive(Deserialize, Debug, Default)]
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
    pub(super) updates: Option<Updates>,
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

  #[derive(Deserialize, Clone, Debug)]
  #[serde(untagged)]
  #[cfg_attr(test, derive(Dummy))]
  pub enum Installs {
    Simple(String),
    Full {
      cmd: String,
      #[serde(default)]
      depends: HashSet<String>,
    },
  }

  #[derive(Deserialize, Clone, Debug)]
  #[serde(untagged)]
  #[cfg_attr(test, derive(Dummy))]
  pub enum Updates {
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
          updates: None,
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
    fn merge(
      mut self,
      Capabilities {
        mut links,
        installs,
        updates,
        depends,
      }: Self,
    ) -> Self {
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
          let cmd_outer: Option<String>;
          let mut depends_outer: HashSet<String> = HashSet::new();

          match installs {
            Installs::Simple(cmd) => cmd_outer = cmd.some(),
            Installs::Full { cmd, depends } => {
              cmd_outer = cmd.some();
              depends_outer = depends;
            }
          }

          *i = match i {
            Installs::Simple(cmd) => Installs::Full {
              cmd: cmd_outer.unwrap_or_else(|| cmd.to_string()),
              depends: depends_outer,
            },
            Installs::Full { cmd, depends } => {
              depends_outer.extend(depends.clone());
              Installs::Full { cmd: cmd_outer.unwrap_or_else(|| cmd.to_string()), depends: depends_outer }
            }
          };
        }
      } else {
        self.installs = installs;
      }

      if let Some(u) = &mut self.updates {
        if let Some(updates) = updates {
          let cmd_outer: Option<String>;
          let mut depends_outer: HashSet<String> = HashSet::new();

          match updates {
            Updates::Simple(cmd) => cmd_outer = cmd.some(),
            Updates::Full { cmd, depends } => {
              cmd_outer = cmd.some();
              depends_outer = depends;
            }
          }

          *u = match u {
            Updates::Simple(cmd) => Updates::Full {
              cmd: cmd_outer.unwrap_or_else(|| cmd.to_string()),
              depends: depends_outer,
            },
            Updates::Full { cmd, depends } => {
              depends_outer.extend(depends.clone());
              Updates::Full { cmd: cmd_outer.unwrap_or_else(|| cmd.to_string()), depends: depends_outer }
            }
          };
        }
      } else {
        self.updates = updates;
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
use crate::FILE_EXTENSION;

#[derive(Clone, Debug)]
pub struct Installs {
  pub(crate) cmd: String,
  pub(crate) depends: HashSet<String>,
}

impl From<repr::Installs> for Installs {
  fn from(from: repr::Installs) -> Self {
    match from {
      repr::Installs::Simple(cmd) => Self { cmd, depends: Default::default() },
      repr::Installs::Full { cmd, depends } => Self { cmd, depends },
    }
  }
}

#[derive(Clone, Debug)]
pub struct Updates {
  pub(crate) cmd: String,
  pub(crate) depends: HashSet<String>,
}

impl From<repr::Updates> for Updates {
  fn from(from: repr::Updates) -> Self {
    match from {
      repr::Updates::Simple(cmd) => Self { cmd, depends: Default::default() },
      repr::Updates::Full { cmd, depends } => Self { cmd, depends },
    }
  }
}

#[derive(Default, Clone, Debug)]
pub struct Dot {
  pub(crate) links: Option<HashMap<PathBuf, HashSet<PathBuf>>>,
  pub(crate) installs: Option<Installs>,
  pub(crate) updates: Option<Updates>,
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

    if let Some(updates) = &merge.updates {
      if let Some(u) = &mut self.updates {
        u.cmd = updates.cmd.clone();
        u.depends.extend(updates.depends.clone());
      } else {
        self.updates = updates.clone().some();
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

impl FromStr for Dot {
  type Err = repr::ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let repr::Dot {
      global,
      windows,
      linux,
      darwin,
      windows_linux,
      linux_darwin,
      darwin_windows,
    } = repr::Dot::parse(s)?;

    let is_windows = cfg!(windows);
    let is_macos = cfg!(target_os = "macos");
    let is_unix = !is_macos && cfg!(unix);

    let mut capabilities: Option<Capabilities> = None;

    if is_windows {
      capabilities = windows.map(|g| *g).merge(global);
      capabilities = capabilities.merge(windows_linux);
      capabilities = capabilities.merge(darwin_windows);
    } else if is_unix {
      capabilities = linux.map(|g| *g).merge(global);
      capabilities = capabilities.merge(windows_linux);
      capabilities = capabilities.merge(linux_darwin);
    } else if is_macos {
      capabilities = darwin.map(|g| *g).merge(global);
      capabilities = capabilities.merge(linux_darwin);
      capabilities = capabilities.merge(darwin_windows);
    }

    if let Some(capabilities) = capabilities {
      Self {
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
        installs: capabilities.installs.map(|i| i.into()),
        updates: capabilities.updates.map(|u| u.into()),
        depends: capabilities.depends.map(|c| c.depends),
      }
    } else {
      Self::default()
    }
    .okay()
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
  let global = dotfiles_path.join(format!("dots.{FILE_EXTENSION}"));
  let global = match fs::read_to_string(global.clone()) {
    Ok(text) => text.parse::<Dot>().map_err(|e| Error::ParseDot(global, e))?.some(),
    Err(err) => match err.kind() {
      std::io::ErrorKind::NotFound => None,
      _ => panic!("{}", err),
    },
  };

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
    Ok((name, Ok(text))) => match text.parse::<Dot>() {
      Ok(dot) => (name, dot).okay().some(),
      Err(err) => Error::ParseDot(Path::new(&format!("{name}/dot.{FILE_EXTENSION}")).to_path_buf(), err).error().some(),
    },
    Ok((_, Err(Error::Io(file, err)))) => match err.kind() {
      std::io::ErrorKind::NotFound => None,
      _ => Error::Io(file, err).error().some(),
    },
    Ok((_, Err(err))) | Err(err) => err.error().some(),
  });

  let dots = dots.map_ok(|d| (d.0, if let Some(global) = &global { global.clone().merge(&d.1) } else { d.1 }));

  let dots = crate::helpers::join_err_result(dots.collect())?;
  if dots.is_empty() {
    println!("Warning: {}", "No dots found".yellow());
    return vec![].okay();
  }

  dots.okay()
}
