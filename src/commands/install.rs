use miette::{Diagnostic, Result};
use somok::Somok;

use crate::config::Config;

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {}

pub struct Install {
  config: Config,
}

impl Install {
  pub const fn new(config: crate::config::Config) -> Self {
    Self { config }
  }
}

impl super::Command for Install {
  type Args = crate::cli::Install;
  type Result = Result<()>;

  fn execute(&self, link_command: Self::Args) -> Self::Result {
    let installs = crate::dot::read_dots(&self.config.dotfiles, &link_command.dots)?
      .into_iter()
      .filter_map(|d| d.1.installs.map(|l| (d.0, l)));

    todo!();

    ().okay()
  }
}
