use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
};

#[cfg(test)]
use fake::Dummy;
use serde::Deserialize;

use super::{DotSimplified, InstallsComplex, LinksComplex};

#[derive(Deserialize, Clone, Default, Debug)]
#[cfg_attr(test, derive(Dummy))]
#[serde(deny_unknown_fields)]
pub(super) struct CapabilitiesComplex {
  pub(super) links: Option<HashMap<PathBuf, LinksComplex>>,
  pub(super) installs: Option<InstallsComplex>,
  pub(super) depends: Option<HashSet<String>>,
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
