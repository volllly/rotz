use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
};

#[cfg(test)]
use fake::Dummy;
use serde::Deserialize;

use super::{InstallsComplex, LinksComplex};

#[derive(Deserialize, Debug, Default)]
#[cfg_attr(test, derive(Dummy))]
#[serde(deny_unknown_fields)]
pub(super) struct DotSimplified {
  pub(super) links: Option<HashMap<PathBuf, LinksComplex>>,
  pub(super) installs: Option<InstallsComplex>,
  pub(super) depends: Option<HashSet<String>>,
}
