use std::path::PathBuf;

use crossterm::style::{Attribute, Stylize};
use git2::{Cred, RemoteCallbacks, Repository};
use miette::{Diagnostic, Result};
use tap::Pipe;
#[cfg(feature = "profiling")]
use tracing::instrument;

use super::Command;
use crate::config::{self, Config};

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("Could not create dotfiles directory \"{0}\"")]
  #[diagnostic(code(init::dotfiles::create))]
  CreatingDir(PathBuf, #[source] std::io::Error),

  #[error("Could not clone git repository \"{0}\"")]
  #[diagnostic(code(init::git::clone))]
  GitClone(PathBuf, #[source] git2::Error),
}

#[derive(Debug)]
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

  #[cfg_attr(feature = "profiling", instrument)]
  fn execute(&self, (cli, repo): Self::Args) -> Self::Result {
    if !cli.dry_run {
      config::create_config_file(cli.dotfiles.as_ref().map(|d| d.0.as_path()), &cli.config.0)?;

      std::fs::create_dir_all(&self.config.dotfiles).map_err(|err| Error::CreatingDir(self.config.dotfiles.clone(), err))?;
    }

    println!(
      "{}Cloning \"{}\" to \"{}\"{}\n",
      Attribute::Bold,
      repo.as_str().blue(),
      self.config.dotfiles.to_string_lossy().green(),
      Attribute::Reset
    );

    if !cli.dry_run {
      clone_git_repo(&repo, &self.config.dotfiles).map_err(|err| Error::GitClone(self.config.dotfiles.clone(), err))?;
      println!("\n{}Cloned repo{}", Attribute::Bold, Attribute::Reset);
    }

    ().pipe(Ok)
  }
}

fn clone_git_repo(repo: &str, dotfiles: &std::path::Path) -> Result<Repository, git2::Error> {
  let mut callbacks = RemoteCallbacks::new();
  callbacks.credentials(|_url, username_from_url, _allowed_types| Cred::ssh_key_from_agent(username_from_url.unwrap()));
  let mut fetch_options = git2::FetchOptions::new();

  fetch_options.remote_callbacks(callbacks);

  let mut builder = git2::build::RepoBuilder::new();
  builder.fetch_options(fetch_options);

  builder.clone(repo, dotfiles)?.pipe(Ok)
}

#[cfg(test)]
mod test {
  use std::path::Path;

  use clap::Parser;
  use speculoos::assert_that;

  use crate::{cli, commands::Command};

  #[test]
  fn should_clone_repo() {
    let config = crate::config::Config {
      dotfiles: Path::new("./target/tmp").to_path_buf(),
      ..Default::default()
    };

    let config_file = config.dotfiles.parent().unwrap().join("config.toml");
    if config_file.exists() {
      std::fs::remove_file(&config_file).unwrap();
    }
    if config.dotfiles.exists() {
      std::fs::remove_dir_all(&config.dotfiles).unwrap();
    }
    let cli = crate::cli::Cli::parse_from([
      "",
      "--dotfiles",
      &config.dotfiles.to_string_lossy(),
      "--config",
      &config_file.to_string_lossy(),
      "clone",
      "git@github.com:volllly/rotz.git",
    ]);

    let clone = super::Clone::new(config.clone());

    let cli::Command::Clone { repo } = cli.command.clone() else {
      panic!();
    };

    clone.execute((cli, repo)).unwrap();
    assert_that!(Path::new(&config.dotfiles).join(".git")).matches(|p| p.exists() && p.is_dir());
  }
}
