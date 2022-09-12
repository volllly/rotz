use std::ffi::OsStr;

use crossterm::style::{Attribute, Stylize};
use miette::{Diagnostic, Result};
use tap::Pipe;

use super::Command;
use crate::{
  config::{self, Config},
  helpers,
};

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
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
  type Args = (crate::cli::Cli, String);

  type Result = Result<()>;

  fn execute(&self, (cli, repo): Self::Args) -> Self::Result {
    if !cli.dry_run {
      config::create_config_file(cli.dotfiles.as_ref().map(|d| d.0.as_path()), &cli.config.0)?;
    }

    println!(
      "{}Cloning \"{}\" to \"{}\"{}\n",
      Attribute::Bold,
      repo.as_str().blue(),
      self.config.dotfiles.to_string_lossy().green(),
      Attribute::Reset
    );

    helpers::run_command("git", &[OsStr::new("clone"), OsStr::new(&repo), self.config.dotfiles.as_os_str()], false, cli.dry_run)?;

    println!("\n{}Cloned repo{}", Attribute::Bold, Attribute::Reset);

    ().pipe(Ok)
  }
}
