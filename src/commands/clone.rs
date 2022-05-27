use std::{io, process::Command};

use crossterm::style::Stylize;
use miette::{miette, Diagnostic, Result};
use somok::Somok;

use crate::config::Config;

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("Git {0} did not complete successfully")]
  #[diagnostic(code(clone::git::generic))]
  GitError(String, #[source] io::Error),
}

pub struct Clone {
  config: Config,
}

impl Clone {
  pub const fn new(config: Config) -> Self {
    Self { config }
  }
}

impl super::Command for Clone {
  type Args = ();

  type Result = Result<()>;

  fn execute(&self, _: Self::Args) -> Self::Result {
    Command::new("git")
      .args([
        "clone",
        self.config.repo.as_ref().ok_or_else(|| miette!("No repo set"))?,
        self.config.dotfiles.as_os_str().to_str().ok_or_else(|| crate::dot::Error::PathParse(self.config.dotfiles.clone()))?,
      ])
      .output()
      .map_err(|e| Error::GitError("clone".to_string(), e))?;

    println!("Cloned {}\n    to {}", self.config.repo.clone().unwrap().blue(), self.config.dotfiles.display().to_string().green());

    ().okay()
  }
}
