use std::{collections::HashSet, path::PathBuf};

#[cfg(test)]
use fake::Dummy;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
#[cfg_attr(test, derive(Dummy))]
#[serde(deny_unknown_fields)]
pub(super) enum LinksComplex {
  One(PathBuf),
  Many(HashSet<PathBuf>),
}
