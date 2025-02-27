use std::collections::HashSet;

#[cfg(test)]
use fake::Dummy;
use serde::Deserialize;
use strum::EnumIs;
use velcro::hash_set;

use super::InstallsComplex;

#[derive(Deserialize, Clone, Debug, EnumIs)]
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
