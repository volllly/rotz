use std::{
  fs,
  path::{Path, PathBuf},
};

use clap::ArgEnum;
use miette::{Result, IntoDiagnostic};
use crossterm::style::Stylize;
use derive_more::{Display, IsVariant};
use serde::{Deserialize, Serialize};
use somok::Somok;

use crate::USER_DIRS;

#[derive(Debug, ArgEnum, Clone, Display, Deserialize, Serialize, IsVariant)]
pub enum LinkType {
  /// Uses symbolic links for linking
  Symbolic,
  /// Uses hard links for linking
  Hard,
}

#[derive(Deserialize, Serialize, Debug)]
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
