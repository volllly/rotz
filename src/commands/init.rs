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
    }

    if !cli.dry_run {
      std::fs::create_dir_all(&self.config.dotfiles).map_err(|err| Error::CreatingDir(self.config.dotfiles.clone(), err))?;
    }

    println!("\n{}Initializing repo in \"{}\"{}\n", Attribute::Bold, self.config.dotfiles.to_string_lossy().green(), Attribute::Reset);

    if !cli.dry_run {
      let mut options = RepositoryInitOptions::new();

      init.repo.as_ref().tap_some(|repo| {
        options.origin_url(repo.as_str());
      });

      let git_repo = Repository::init_opts(&self.config.dotfiles, &options).map_err(|err| Error::GitInit(self.config.dotfiles.clone(), err))?;

      let sig = git_repo.signature().unwrap();
      let tree_id = {
        let mut index = git_repo.index().unwrap();

        index.write_tree().unwrap()
      };

      let tree = git_repo.find_tree(tree_id).unwrap();
      git_repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();

      init.repo.tap_some(|repo| {
        let head = git_repo.head().unwrap();
        if head.is_branch() {
          git_repo.find_remote(repo).unwrap().push(&[head.name().unwrap()], None).unwrap();
        }
      });

      println!("\n{}Initialized repo{}", Attribute::Bold, Attribute::Reset);
    };

    Ok(())
  }
}
