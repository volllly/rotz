use std::collections::HashSet;

#[cfg(test)]
use fake::Dummy;
use serde::Deserialize;
use strum::EnumIs;

#[derive(Deserialize, Clone, Debug, EnumIs)]
#[serde(untagged)]
#[cfg_attr(test, derive(Dummy))]
#[serde(deny_unknown_fields)]
pub(super) enum InstallsComplex {
  None(bool),
  Simple(String),
  Full {
    cmd: String,
    #[serde(default)]
    depends: HashSet<String>,
  },
}
