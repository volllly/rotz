use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
};

#[cfg(test)]
use fake::Dummy;
use itertools::{Either, Itertools};
use serde::Deserialize;
#[cfg(feature = "profiling")]
use tracing::instrument;
use velcro::hash_set;

use crate::templating::{Engine, Parameters};

use super::{CapabilitiesComplex, DotCanonical, InstallsCanonical, LinksComplex, Merge};

#[derive(Deserialize, Clone, Default, Debug)]
#[cfg_attr(test, derive(Dummy))]
#[serde(deny_unknown_fields)]
pub struct CapabilitiesCanonical {
  pub links: Option<HashMap<PathBuf, HashSet<PathBuf>>>,
  pub installs: Option<InstallsCanonical>,
  pub depends: Option<HashSet<String>>,
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

impl CapabilitiesCanonical {
  #[cfg_attr(feature = "profiling", instrument(skip(engine)))]
  pub fn from(DotCanonical { filters }: DotCanonical, engine: &Engine<'_>, parameters: &Parameters<'_>) -> Self {
    let (globals, filters): (Vec<_>, Vec<_>) = filters
      .into_iter()
      .filter(|(filter, _)| filter.applies(engine, parameters))
      .partition_map(|(filter, capability)| if filter.is_global() { Either::Left } else { Either::Right }(capability));
    let mut capabilities = None::<CapabilitiesCanonical>;

    for capability in globals {
      capabilities = capabilities.merge(capability.into());
    }

    for capability in filters {
      capabilities = capabilities.merge(capability.into());
    }

    capabilities.unwrap_or_default()
  }
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
