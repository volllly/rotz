use std::{io, path::PathBuf, process::Command};

use crossterm::style::Stylize;
use miette::{miette, Diagnostic, Result};
use somok::Somok;

use crate::config::Config;

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("Invalid dotfiles path {0}")]
  InvalidPath(PathBuf),

  #[error("Git {0} did not complete successfully")]
  GitError(String, #[source] io::Error),
}

pub fn execute(Config { dotfiles, link_type: _, repo }: Config) -> Result<()> {
  Command::new("git")
    .args([
      "clone",
      repo.as_ref().ok_or_else(|| miette!("No repo set"))?,
      dotfiles.as_os_str().to_str().ok_or_else(|| Error::InvalidPath(dotfiles.clone()))?,
    ])
    .output()
    .map_err(|e| Error::GitError("clone".to_string(), e))?;

  println!("Cloned {}\n    to {}", repo.unwrap().blue(), dotfiles.display().to_string().green());

  ().okay()
}
