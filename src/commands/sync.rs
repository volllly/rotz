use std::ffi::OsStr;

use crossterm::style::Attribute;
use miette::{Diagnostic, Result};
use somok::Somok;

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

  fn execute(&self, (globals, sync): Self::Args) -> Self::Result {
    if !sync.no_push {
      println!("{}Adding files{}\n", Attribute::Bold, Attribute::Reset);
      if sync.dots.contains(&"*".to_string()) {
        helpers::run_command(
          "git",
          &[OsStr::new("-C"), self.config.dotfiles.as_os_str(), OsStr::new("add"), OsStr::new("*"), OsStr::new("-v")],
          true,
          globals.dry_run,
        )
        .map_err(|err| Error::CommandExecute("Add *".to_string(), err))?;
      } else {
        for dot in sync.dots {
          helpers::run_command(
            "git",
            &[OsStr::new("-C"), self.config.dotfiles.as_os_str(), OsStr::new("add"), OsStr::new(&format!("{dot}/*")), OsStr::new("-v")],
            true,
            globals.dry_run,
          )
          .map_err(|err| Error::CommandExecute("Add".to_string(), err))?;
        }
      }
    }

    println!("{}Commiting{}\n", Attribute::Bold, Attribute::Reset);
    helpers::run_command(
      "git",
      &[
        OsStr::new("-C"),
        self.config.dotfiles.as_os_str(),
        OsStr::new("commit"),
        OsStr::new("-m"),
        OsStr::new(&sync.message.unwrap_or_else(|| "rotz sync".to_string())),
      ],
      true,
      globals.dry_run,
    )
    .map_err(|err| Error::CommandExecute("Commit".to_string(), err))?;

    println!("{}Pulling{}\n", Attribute::Bold, Attribute::Reset);
    helpers::run_command("git", &[OsStr::new("-C"), self.config.dotfiles.as_os_str(), OsStr::new("pull")], true, globals.dry_run).map_err(|err| Error::CommandExecute("Pull".to_string(), err))?;

    if !sync.no_push {
      println!("{}Pushing{}\n", Attribute::Bold, Attribute::Reset);
      helpers::run_command("git", &[OsStr::new("-C"), self.config.dotfiles.as_os_str(), OsStr::new("push")], true, globals.dry_run).map_err(|err| Error::CommandExecute("Push".to_string(), err))?;
    }

    println!("{}Sync complete{}\n", Attribute::Bold, Attribute::Reset);

    ().okay()
  }
}
