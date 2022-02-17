use std::path::PathBuf;

use clap::ArgEnum;
use derive_more::{Display, IsVariant};
use serde::{Deserialize, Serialize};

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
