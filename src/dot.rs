mod repr {
  use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
  };

  use serde::Deserialize;
  use somok::Somok;

  #[cfg(test)]
  use fake::{Dummy, Fake};

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
  #[cfg_attr(test, derive(Dummy))]
  pub struct Installs {
    pub cmd: String,
    pub depends: HashSet<String>,
  }

  #[derive(Deserialize, Clone, Debug)]
  #[cfg_attr(test, derive(Dummy))]
  pub struct Updates {
    pub cmd: String,
    pub depends: HashSet<String>,
  }

  #[derive(Deserialize, Clone, Debug)]
  #[cfg_attr(test, derive(Dummy))]
  pub struct Depends {
    #[serde(flatten)]
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
    pub fn parse(value: &str) -> Result<Self, ParseError> {
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
        parse_inner::<Dot>(value)
      } else {
        Dot {
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

  impl Merge<Capabilities> for Capabilities {
    fn merge(
      mut self,
      Capabilities {
        mut links,
        installs,
        updates,
        depends,
      }: Capabilities,
    ) -> Self {
      if let Some(self_links) = &mut self.links {
        if let Links::One { links: self_links_one } = self_links {
          *self_links = Links::Many {
            links: self_links_one
              .iter_mut()
              .map(|l| {
                let mut hs = HashSet::new();
                hs.insert(l.1.to_owned());
                (l.0.to_owned(), hs)
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
                hs.insert(l.1.to_owned());
                (l.0.to_owned(), hs)
              })
              .collect(),
          };
        }
      }
      if let Some(self_links) = &mut self.links {
        if let Some(merge_links) = &mut links {
          if let Links::Many { links: self_links_many } = self_links {
            if let Links::Many { links: merge_links_many } = merge_links {
              merge_links_many.iter_mut().for_each(|l| {
                if !self_links_many.contains_key(l.0) {
                  self_links_many.insert(l.0.to_owned(), l.1.to_owned());
                } else {
                  let self_links_many_value = self_links_many.get_mut(l.0).unwrap();
                  self_links_many_value.extend(l.1.to_owned());
                }
              })
            }
          }
        }
      } else {
        self.links = links;
      }

      if let Some(i) = &mut self.installs {
        if let Some(installs) = installs {
          i.cmd = installs.cmd;
          i.depends.extend(installs.depends);
        }
      } else {
        self.installs = installs;
      }

      if let Some(u) = &mut self.updates {
        if let Some(updates) = updates {
          u.cmd = updates.cmd;
          u.depends.extend(updates.depends);
        }
      } else {
        self.updates = updates;
      }

      if let Some(d) = &mut self.depends {
        if let Some(depends) = depends {
          d.depends.extend(depends.depends)
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
  path::PathBuf,
  str::FromStr,
};

pub use repr::{Installs, Merge, Updates};
use somok::Somok;

use self::repr::Capabilities;

#[derive(Default, Clone, Debug)]
pub struct Dot {
  pub links: Option<HashMap<PathBuf, HashSet<PathBuf>>>,
  pub installs: Option<Installs>,
  pub updates: Option<Updates>,
  pub depends: Option<HashSet<String>>,
}

impl Merge<&Dot> for Dot {
  fn merge(mut self, merge: &Dot) -> Self {
    if let Some(links) = &merge.links {
      if let Some(l) = &mut self.links {
        l.extend(links.clone())
      } else {
        self.links = links.to_owned().some()
      }
    }

    if let Some(installs) = &merge.installs {
      if let Some(i) = &mut self.installs {
        i.cmd = installs.cmd.clone();
        i.depends.extend(installs.depends.clone());
      } else {
        self.installs = installs.to_owned().some()
      }
    }

    if let Some(updates) = &merge.updates {
      if let Some(u) = &mut self.updates {
        u.cmd = updates.cmd.clone();
        u.depends.extend(updates.depends.clone());
      } else {
        self.updates = updates.to_owned().some()
      }
    }

    if let Some(depends) = &merge.depends {
      if let Some(d) = &mut self.depends {
        d.extend(depends.clone())
      } else {
        self.depends = depends.to_owned().some()
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
        installs: capabilities.installs,
        updates: capabilities.updates,
        depends: capabilities.depends.map(|c| c.depends),
      }
    } else {
      Dot::default()
    }
    .okay()
  }
}
