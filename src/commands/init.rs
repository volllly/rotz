use std::{fmt::Debug, path::PathBuf};

use crossterm::style::{Attribute, Stylize};
use git2::{Repository, RepositoryInitOptions};
use miette::{Diagnostic, Result};
use tap::TapOptional;
#[cfg(feature = "profiling")]
use tracing::instrument;

use super::Command;
use crate::config::{self, Config};

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("Could not create dotfiles directory \"{0}\"")]
  #[diagnostic(code(init::dotfiles::create))]
  CreatingDir(PathBuf, #[source] std::io::Error),

  #[error("Could not initialize git repository \"{0}\"")]
  #[diagnostic(code(init::git::init))]
  GitInit(PathBuf, #[source] git2::Error),
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
  type Args = (crate::cli::Cli, crate::cli::InitRaw);

  type Result = Result<()>;

  #[cfg_attr(feature = "profiling", instrument)]
  fn execute(&self, (cli, init): Self::Args) -> Self::Result {
    if !cli.dry_run {
      config::create_config_file(cli.dotfiles.as_ref().map(|d| d.0.as_path()), &cli.config.0)?;

      std::fs::create_dir_all(&self.config.dotfiles).map_err(|err| Error::CreatingDir(self.config.dotfiles.clone(), err))?;
    }

    println!("\n{}Initializing repo in \"{}\"{}\n", Attribute::Bold, self.config.dotfiles.to_string_lossy().green(), Attribute::Reset);

    if !cli.dry_run {
      let mut options = RepositoryInitOptions::new();

      init.repo.as_ref().tap_some(|repo| {
        options.origin_url(repo.as_str());
      });

      options.initial_head("main");

      initialize_git_repo(&self.config.dotfiles, &options).map_err(|err| Error::GitInit(self.config.dotfiles.clone(), err))?;
      println!("\n{}Initialized repo{}", Attribute::Bold, Attribute::Reset);
    };

    Ok(())
  }
}

fn initialize_git_repo(dotfiles: &PathBuf, options: &RepositoryInitOptions) -> Result<(), git2::Error> {
  let git_repo = Repository::init_opts(dotfiles, options)?;

  let sig = git_repo.signature()?;
  let tree_id = {
    let mut index = git_repo.index()?;

    index.write_tree()?
  };

  let tree = git_repo.find_tree(tree_id)?;
  git_repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

  Ok(())
}

#[cfg(test)]
mod test {
  use std::path::Path;

  use clap::Parser;
  use speculoos::assert_that;

  use crate::{cli, commands::Command};

  #[test]
  fn should_initialize_repo() {
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
      "init",
      "git@github.com:volllly/rotz.git",
    ]);

    let init = super::Init::new(config.clone());

    let cli::Command::Init { init: command } = cli.command.clone() else {
      panic!();
    };

    init.execute((cli, command)).unwrap();
    assert_that!(Path::new(&config.dotfiles).join(".git")).matches(|p| p.exists() && p.is_dir());
  }
}
