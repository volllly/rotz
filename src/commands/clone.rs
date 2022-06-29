use std::{io, process};

use crossterm::style::Stylize;
use miette::{Diagnostic, Result};
use somok::Somok;

use super::Command;
use crate::config::Config;

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("No repo is configured")]
  #[diagnostic(code(clone::config::repo), help("Run the clone command with the --repo argument"))]
  NoRepoConfigured,

  #[error("Cloud not spawn git {0} command")]
  #[diagnostic(code(clone::command::spawn))]
  GitSpawn(String, #[source] io::Error),

  #[error("Git {0} did not complete successfully. (Exitcode {1:?})")]
  #[diagnostic(code(clone::command::execute))]
  GitExecute(String, Option<i32>),

  #[error(transparent)]
  PathParse(#[from] crate::dot::Error),
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
    if !globals.dry_run {
      let output = process::Command::new("git")
        .args([
          "clone",
          self.config.repo.as_ref().ok_or(Error::NoRepoConfigured)?,
          self
            .config
            .dotfiles
            .as_os_str()
            .to_str()
            .ok_or_else(|| Error::from(crate::dot::Error::PathParse(self.config.dotfiles.clone())))?,
        ])
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .output()
        .map_err(|e| Error::GitSpawn("clone".to_string(), e))?;

      if !output.status.success() {
        Error::GitExecute("clone".to_string(), output.status.code()).error()?;
      }
    }

    println!("Cloned {}\n    to {}", self.config.repo.clone().unwrap().blue(), self.config.dotfiles.display().to_string().green());

    ().okay()
  }
}
