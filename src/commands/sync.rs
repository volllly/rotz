use std::ffi::OsStr;

use crossterm::style::Attribute;
use miette::{Diagnostic, Result};
use tap::Pipe;
#[cfg(feature = "profiling")]
use tracing::instrument;
use walkdir::WalkDir;
use wax::Pattern;

use super::Command;
use crate::{config::Config, helpers};

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error(transparent)]
  PathParse(#[from] crate::dot::Error),

  #[error("{0} command did not run successfully")]
  #[diagnostic(code(clone::command::run))]
  CommandExecute(String, #[source] helpers::RunError),
}

#[derive(Debug)]
pub struct Sync {
  config: Config,
}

impl Sync {
  pub const fn new(config: Config) -> Self {
    Self { config }
  }
}

impl Command for Sync {
  type Args = (crate::cli::Globals, crate::cli::Sync);

  type Result = Result<()>;

  #[cfg_attr(feature = "profiling", instrument)]
  fn execute(&self, (globals, sync): Self::Args) -> Self::Result {
    println!("{}Adding files{}", Attribute::Bold, Attribute::Reset);
    let globs = helpers::glob_from_vec(&sync.dots, None)?;

    for entry in WalkDir::new(&self.config.dotfiles)
      .into_iter()
      .filter_map(Result::ok)
      .filter(|e| !e.path().starts_with(".git"))
      .filter(|e| globs.is_match(e.path()))
      .filter(|e| !e.file_type().is_dir())
    {
      helpers::run_command(
        "git",
        &[OsStr::new("-C"), self.config.dotfiles.as_os_str(), OsStr::new("add"), OsStr::new(entry.path()), OsStr::new("-v")],
        true,
        globals.dry_run,
      )
      .map_err(|err| Error::CommandExecute("Add".to_owned(), err))?;
    }

    println!("\n{}Commiting{}", Attribute::Bold, Attribute::Reset);
    helpers::run_command(
      "git",
      &[
        OsStr::new("-C"),
        self.config.dotfiles.as_os_str(),
        OsStr::new("commit"),
        OsStr::new("-m"),
        OsStr::new(&sync.message.unwrap_or_else(|| "rotz sync".to_owned())),
      ],
      true,
      globals.dry_run,
    )
    .map_err(|err| Error::CommandExecute("Commit".to_owned(), err))?;

    println!("\n{}Pulling{}", Attribute::Bold, Attribute::Reset);
    helpers::run_command("git", &[OsStr::new("-C"), self.config.dotfiles.as_os_str(), OsStr::new("pull")], true, globals.dry_run).map_err(|err| Error::CommandExecute("Pull".to_owned(), err))?;

    if !sync.no_push {
      println!("\n{}Pushing{}", Attribute::Bold, Attribute::Reset);
      helpers::run_command("git", &[OsStr::new("-C"), self.config.dotfiles.as_os_str(), OsStr::new("push")], true, globals.dry_run).map_err(|err| Error::CommandExecute("Push".to_owned(), err))?;
    }

    println!("\n{}Sync complete{}", Attribute::Bold, Attribute::Reset);

    ().pipe(Ok)
  }
}
