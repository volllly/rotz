use std::{
  fs,
  path::{Path, PathBuf},
};

use clap::ArgEnum;
use crossterm::style::Stylize;
use derive_more::{Display, IsVariant};
#[cfg(test)]
use fake::{Dummy, Fake};
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use somok::Somok;

use crate::USER_DIRS;

#[derive(Debug, ArgEnum, Clone, Display, Deserialize, Serialize, IsVariant)]
#[cfg_attr(test, derive(Dummy, PartialEq))]
pub enum LinkType {
  /// Uses symbolic links for linking
  Symbolic,
  /// Uses hard links for linking
  Hard,
}

#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(test, derive(Dummy, PartialEq))]
pub struct Config {
  /// Path to the local dotfiles
  pub dotfiles: PathBuf,
  /// Which link type to use for linking dotfiles
  pub link_type: LinkType,
  /// The url of the repository passed to the git clone command
  pub repo: Option<String>,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      dotfiles: USER_DIRS.home_dir().join(".dotfiles"),
      link_type: LinkType::Symbolic,
      repo: None,
    }
  }
}

#[cfg(feature = "yaml")]
fn deserialize_config(config: &str) -> Result<Config, serde_yaml::Error> {
  serde_yaml::from_str(config)
}

#[cfg(feature = "yaml")]
fn serialize_config(config: &Config) -> Result<String, serde_yaml::Error> {
  serde_yaml::to_string(&config)
}

#[cfg_attr(all(nightly, coverage), no_coverage)]
pub fn create_config_file_with_repo(repo: &str, config_file: &Path) -> Result<()> {
  let mut config = Config::default();
  if let Ok(existing_config_str) = fs::read_to_string(config_file) {
    if let Ok(existing_config) = deserialize_config(&existing_config_str) {
      if existing_config.repo.as_ref().map(|r| *r != *repo).unwrap_or(false) {
        println!("Warning: {}", "Config file already exists and contains a different repo".yellow());
        return ().okay();
      }
      config = existing_config;
    }
  }

  config.repo = repo.to_string().some();

  fs::write(config_file, serialize_config(&config).into_diagnostic()?).into_diagnostic()?;

  ().okay()
}

#[cfg(test)]
mod tests {
  use fake::{Fake, Faker};
  use rstest::rstest;
  use speculoos::prelude::*;

  use super::Config;

  #[rstest]
  #[case(Faker.fake::<Config>())]
  #[case(Config::default())]
  fn ser_de(#[case] config: Config) {
    let serialized = super::serialize_config(&config);
    assert_that!(&serialized).is_ok();
    let serialized = serialized.unwrap();

    let deserialized = super::deserialize_config(&serialized);
    assert_that!(&deserialized).is_ok().is_equal_to(config);
  }
}
