use crossterm::style::{Attribute, Stylize};
use miette::{Diagnostic, Result};
use somok::Somok;

use super::Command;
use crate::{config::Config, helpers};

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("No repo is configured")]
  #[diagnostic(code(clone::config::repo), help("Run the clone command with the --repo argument"))]
  NoRepoConfigured,

  #[error(transparent)]
  PathParse(#[from] crate::dot::Error),

  #[error("Clone command did not run successfully")]
  #[diagnostic(code(clone::command::run))]
  CloneExecute(#[from] helpers::RunError),
}

pub struct Clone {
  config: Config,
}

impl Clone {
  pub const fn new(config: Config) -> Self {
    Self { config }
  }
}

impl Command for Clone {
  type Args = crate::cli::Globals;

  type Result = Result<()>;

  fn execute(&self, globals: Self::Args) -> Self::Result {
    let repo = self.config.repo.as_ref().ok_or(Error::NoRepoConfigured)?;
    let path = self
      .config
      .dotfiles
      .as_os_str()
      .to_str()
      .ok_or_else(|| Error::from(crate::dot::Error::PathParse(self.config.dotfiles.clone())))?;

    println!("{}Cloning \"{}\" to \"{}\"{}\n", Attribute::Bold, repo.as_str().blue(), path.blue(), Attribute::Reset);

    helpers::run_command("git", &["clone", repo, path], false, globals.dry_run)?;

    println!("Cloned {}\n    to {}", self.config.repo.clone().unwrap().blue(), self.config.dotfiles.display().to_string().green());

    ().okay()
  }
}
