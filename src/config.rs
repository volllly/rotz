use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

use clap::ArgEnum;
use crossterm::style::Stylize;
use derive_more::{Display, IsVariant};
#[cfg(test)]
use fake::{Dummy, Fake};
use miette::{Diagnostic, Result};
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
  pub(crate) dotfiles: PathBuf,

  /// Which link type to use for linking dotfiles
  pub(crate) link_type: LinkType,

  /// The url of the repository passed to the git clone command
  pub(crate) repo: Option<String>,

  /// The command used to spawn processess.
  /// Use handlebars templates `{{ cmd }}` as placeholder for the cmd set in the dot.
  /// E.g. `"bash -c {{ quote "" cmd }}"`.
  pub(crate) shell_command: Option<String>,

  /// Variables can be used for templating in dot.(yaml|toml|json) files.
  pub(crate) variables: HashMap<String, String>,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      dotfiles: USER_DIRS.home_dir().join(".dotfiles"),
      link_type: LinkType::Symbolic,
      repo: None,
      #[cfg(windows)]
      shell_command: Some("pwsh -NoProfile -C {{ quote \"\" cmd }}".to_string()),
      #[cfg(all(not(target_os = "macos"), unix))]
      shell_command: Some("bash -c {{ quote \"\" cmd }}".to_string()),
      #[cfg(target_os = "macos")]
      shell_command: Some("zsh -c {{ quote \"\" cmd }}".to_string()),
      variables: HashMap::new(),
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

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum Error {
  #[cfg(feature = "yaml")]
  #[error("Could not serialize config")]
  #[diagnostic(code(config::serialize))]
  SerializingConfig(#[source] serde_yaml::Error),

  #[error("Could not write config")]
  #[diagnostic(code(config::write))]
  WritingConfig(PathBuf, #[source] std::io::Error),
}

#[cfg_attr(all(nightly, coverage), no_coverage)]
pub fn create_config_file_with_repo(repo: &str, config_file: &Path) -> Result<()> {
  let mut config = Config::default();
  if let Ok(existing_config_str) = fs::read_to_string(config_file) {
    if let Ok(existing_config) = deserialize_config(&existing_config_str) {
      if existing_config.repo.as_ref().map_or(false, |r| *r != *repo) {
        println!("Warning: {}", "Config file already exists and contains a different repo".yellow());
        return ().okay();
      }
      config = existing_config;
    }
  }

  config.repo = repo.to_string().some();

  fs::write(config_file, serialize_config(&config).map_err(Error::SerializingConfig)?).map_err(|e| Error::WritingConfig(config_file.to_path_buf(), e))?;

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
