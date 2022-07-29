use miette::{Diagnostic, Result};
use somok::Somok;

use super::Command;
use crate::config::{self, Config};

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error(transparent)]
  PathParse(#[from] crate::dot::Error),
}

pub struct Init {
  config: Config,
}

impl Init {
  pub const fn new(config: Config) -> Self {
    Self { config }
  }
}

impl Command for Init {
  type Args = crate::cli::Cli;

  type Result = Result<()>;

  fn execute(&self, cli: Self::Args) -> Self::Result {
    if !cli.dry_run {
      config::create_config_file(self.config.repo.as_deref(), cli.dotfiles.as_ref().map(|d| d.0.as_path()), &cli.config.0)?;
    }

    ().okay()
  }
}
