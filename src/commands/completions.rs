use clap::CommandFactory;
use clap_complete::{Shell, generate};

use super::Command;
use miette::{Diagnostic, Result};

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("Could not determine shell. Please add the shell option")]
  NoShell,
}

#[derive(Debug)]
pub struct Completions {}

impl Completions {
  pub const fn new() -> Self {
    Self {}
  }
}

impl Command for Completions {
  type Args = Option<Shell>;
  type Result = Result<()>;

  #[cfg_attr(feature = "profiling", tracing::instrument)]
  fn execute(&self, shell: Self::Args) -> Self::Result {
    let mut command = crate::cli::Cli::command();

    let shell = shell.or_else(Shell::from_env);

    let cmd_name = command.get_name().to_owned();
    shell.map_or_else(
      || Err(Error::NoShell.into()),
      |shell| {
        generate(shell, &mut command, cmd_name, &mut std::io::stdout());
        Ok(())
      },
    )
  }
}
