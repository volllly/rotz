use std::{ffi::OsStr, fmt::Debug, path::PathBuf};

use crossterm::style::{Attribute, Stylize};
use miette::{Diagnostic, Result};
use tap::Pipe;
#[cfg(feature = "profiling")]
use tracing::instrument;

use super::Command;
use crate::{
  config::{self, Config},
  helpers,
};

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("Could not create dotfiles directory \"{0}\"")]
  #[diagnostic(code(init::dotfiles::create))]
  CreatingDir(PathBuf, #[source] std::io::Error),
}

#[derive(Debug)]
pub struct Init {
  config: Config,
}

impl Init {
  pub const fn new(config: Config) -> Self {
    Self { config }
  }
}

impl Command for Init {
  type Args = (crate::cli::Cli, Option<String>);

  type Result = Result<()>;

  #[cfg_attr(feature = "profiling", instrument)]
  fn execute(&self, (cli, repo): Self::Args) -> Self::Result {
    if !cli.dry_run {
      config::create_config_file(cli.dotfiles.as_ref().map(|d| d.0.as_path()), &cli.config.0)?;
    }

    std::fs::create_dir_all(&self.config.dotfiles).map_err(|err| Error::CreatingDir(self.config.dotfiles.clone(), err))?;

    println!("\n{}Initializing repo in \"{}\"{}\n", Attribute::Bold, self.config.dotfiles.to_string_lossy().green(), Attribute::Reset);

    helpers::run_command(
      "git",
      &[OsStr::new("-C"), self.config.dotfiles.as_os_str(), OsStr::new("init"), OsStr::new("-b"), OsStr::new("main")],
      false,
      cli.dry_run,
    )?;

    if let Some(repo) = repo.as_ref() {
      println!("\n{}Adding remote \"{}\"{}\n", Attribute::Bold, repo.as_str().blue(), Attribute::Reset);

      helpers::run_command(
        "git",
        &[
          OsStr::new("-C"),
          self.config.dotfiles.as_os_str(),
          OsStr::new("remote"),
          OsStr::new("add"),
          OsStr::new("origin"),
          OsStr::new(repo),
        ],
        false,
        cli.dry_run,
      )?;

      helpers::run_command(
        "git",
        &[
          OsStr::new("-C"),
          self.config.dotfiles.as_os_str(),
          OsStr::new("push"),
          OsStr::new("--set-upstream"),
          OsStr::new("origin"),
          OsStr::new("main"),
        ],
        false,
        cli.dry_run,
      )?;

      helpers::run_command(
        "git",
        &[OsStr::new("-C"), self.config.dotfiles.as_os_str(), OsStr::new("push"), OsStr::new("-u"), OsStr::new("origin")],
        false,
        cli.dry_run,
      )?;

      println!("\n{}Initialized repo{}", Attribute::Bold, Attribute::Reset);
    }
    ().pipe(Ok)
  }
}
