use std::fmt::Display;

use baker::Bake;
use clap::{Args, Parser, Subcommand};
use derive_more::{From, FromStr, Into};
#[cfg(test)]
use fake::{Dummy, Fake};
use figment::{
  map,
  value::{Dict, Map, Value},
  Error, Metadata, Profile, Provider,
};
use somok::Somok;

use crate::{config::LinkType, FILE_EXTENSION, PROJECT_DIRS};

#[derive(From, Debug, FromStr, Into)]
#[cfg_attr(test, derive(Dummy, PartialEq))]
pub struct PathBuf(pub(crate) std::path::PathBuf);

impl Display for PathBuf {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0.display())
  }
}

#[derive(Parser, Debug, Bake)]
#[clap(version, about)]
#[cfg_attr(test, derive(Dummy, PartialEq))]
#[baked(name = "Globals")]
pub struct Cli {
  #[clap(long, short)]
  #[baked(ignore)]
  /// Overwrites the dotfiles path set in the config file
  ///
  /// If no dotfiles path is provided in the config file the default "~/.dotfiles" is used
  pub(crate) dotfiles: Option<PathBuf>,

  #[clap(long, short, default_value_t = PROJECT_DIRS.config_dir().join(format!("config.{FILE_EXTENSION}")).into())]
  #[baked(ignore)]
  /// Path to the config file
  pub(crate) config: PathBuf,

  #[clap(long, short = 'r')]
  /// When this switch is set no changes will be made.
  pub(crate) dry_run: bool,

  #[clap(subcommand)]
  #[baked(ignore)]
  pub(crate) command: Command,
}

#[derive(Debug, Args, Clone)]
#[cfg_attr(test, derive(Dummy, PartialEq))]
pub struct Dots {
  #[clap(default_value = "*")]
  /// All dots to process
  pub(crate) dots: Vec<String>,
}

#[derive(Debug, Args, Bake, Clone)]
#[cfg_attr(test, derive(Dummy, PartialEq))]
#[baked(name = "Link")]
pub struct LinkCli {
  #[clap(flatten)]
  #[baked(type = "Vec<String>", map = "self.dots.dots")]
  pub(crate) dots: Dots,

  #[clap(long, short)]
  /// Force link creation if file already exists
  pub(crate) force: bool,

  #[clap(long, short, arg_enum)]
  #[baked(ignore)]
  /// Which link type to use for linking dotfiles
  link_type: Option<LinkType>,
}

#[derive(Debug, Args, Bake, Clone)]
#[cfg_attr(test, derive(Dummy, PartialEq))]
#[baked(name = "Install")]
pub struct InstallCli {
  #[clap(flatten)]
  #[baked(type = "Vec<String>", map = "self.dots.dots")]
  pub(crate) dots: Dots,

  /// Continues installation when an error occurs during installation
  #[clap(long, short)]
  pub(crate) continue_on_error: bool,

  /// Do not install dependencies
  #[clap(long, short = 'd')]
  pub(crate) skip_dependencies: bool,

  /// Do not install installation dependencies
  #[clap(long, short = 'i')]
  pub(crate) skip_installation_dependencies: bool,

  /// Do not install any dependencies
  #[clap(long, short = 'a')]
  pub(crate) skip_all_dependencies: bool,
}

#[derive(Debug, Args, Bake, Clone)]
#[cfg_attr(test, derive(Dummy, PartialEq))]
#[baked(name = "Sync")]
pub struct SyncCli {
  #[clap(flatten)]
  #[baked(type = "Vec<String>", map = "self.dots.dots")]
  pub(crate) dots: Dots,

  #[clap(long, short)]
  /// Do not push changes
  pub(crate) no_push: bool,

  #[clap(long, short)]
  /// Specify commit message
  pub(crate) message: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
#[cfg_attr(test, derive(Dummy, PartialEq))]
pub enum Command {
  /// Clones a dotfiles git repository
  Clone {
    /// The url of the repository passed to the git clone command
    repo: String,
  },

  /// Creates a dotfiles git repository and config
  Init {
    /// The url of the repository passed to the git init command
    repo: Option<String>,
  },

  /// Links dotfiles to the filesystem
  Link {
    #[clap(flatten)]
    link: LinkCli,
  },

  /// Syncs dotfiles with the git repository
  Sync {
    #[clap(flatten)]
    sync: SyncCli,
  },

  /// Installs applications using the provided commands
  Install {
    #[clap(flatten)]
    install: InstallCli,
  },
}

impl Provider for Cli {
  fn metadata(&self) -> Metadata {
    Metadata::named("Cli")
  }

  fn data(&self) -> Result<Map<Profile, Dict>, Error> {
    let mut dict = Dict::new();

    if let Some(dotfiles) = &self.dotfiles {
      dict.insert("dotfiles".to_string(), Value::serialize(dotfiles.to_string())?);
    }

    if let Command::Link {
      link: LinkCli {
        dots: _,
        force: _,
        link_type: Some(link_type),
      },
    } = &self.command
    {
      dict.insert("link_type".to_string(), Value::serialize(link_type)?);
    }

    map! {
      Profile::Global => dict
    }
    .okay()
  }
}
