use std::{fmt::Display, str::FromStr};

use baker::Bake;
use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;
#[cfg(test)]
use fake::Dummy;
use figment::{
  Error, Metadata, Profile, Provider, map,
  value::{Dict, Map, Value},
};
use itertools::Itertools;
use tap::Pipe;
#[cfg(feature = "profiling")]
use tracing::instrument;

use crate::{FILE_EXTENSIONS, PROJECT_DIRS, config::LinkType, helpers};

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Dummy, PartialEq, Eq))]
pub struct PathBuf(pub(crate) std::path::PathBuf);

impl From<std::path::PathBuf> for PathBuf {
  fn from(value: std::path::PathBuf) -> Self {
    Self(value)
  }
}

impl FromStr for PathBuf {
  type Err = <std::path::PathBuf as FromStr>::Err;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    PathBuf(std::path::PathBuf::from_str(s)?).pipe(Ok)
  }
}

impl Display for PathBuf {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0.display())
  }
}

#[derive(Parser, Debug, Bake)]
#[clap(version, about)]
#[cfg_attr(test, derive(Dummy, PartialEq, Eq))]
#[baked(name = "Globals", derive(Debug))]
pub struct Cli {
  #[clap(long, short)]
  #[baked(ignore)]
  /// Overwrites the dotfiles path set in the config file
  ///
  /// If no dotfiles path is provided in the config file the default "~/.dotfiles" is used
  pub(crate) dotfiles: Option<PathBuf>,

  #[clap(long, short, default_value_t = {
    helpers::get_file_with_format(PROJECT_DIRS.config_dir(), "config")
      .map(|p| p.0)
      .unwrap_or_else(|| PROJECT_DIRS.config_dir().join(format!("config.{}", FILE_EXTENSIONS[0].0)))
      .into()
})]
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
#[cfg_attr(test, derive(Dummy, PartialEq, Eq))]
pub struct Dots {
  #[clap(default_value = "**")]
  /// All dots to process. Accepts glob patterns.
  pub(crate) dots: Vec<String>,
}

impl Dots {
  #[cfg_attr(feature = "profiling", instrument)]
  fn add_root(&self) -> Self {
    Self {
      dots: self.dots.iter().map(|d| if d.starts_with('/') { d.to_string() } else { format!("/{d}") }).collect_vec(),
    }
  }
}

#[derive(Debug, Args, Bake, Clone)]
#[cfg_attr(test, derive(Dummy, PartialEq, Eq))]
#[baked(name = "Link", derive(Debug))]
pub struct LinkRaw {
  #[clap(flatten)]
  #[baked(type = "Vec<String>", map_fn(bake = "|l| l.dots.add_root().dots"))]
  pub(crate) dots: Dots,

  #[clap(long, short)]
  /// Force link creation if file already exists and was not created by rotz
  pub(crate) force: bool,

  #[clap(long, short)]
  #[baked(ignore)]
  /// Which link type to use for linking dotfiles
  link_type: Option<LinkType>,
}

#[derive(Debug, Args, Bake, Clone)]
#[cfg_attr(test, derive(Dummy, PartialEq, Eq))]
#[baked(name = "Install", derive(Debug))]
#[allow(clippy::struct_excessive_bools)]
pub struct InstallRaw {
  #[clap(flatten)]
  #[baked(type = "Vec<String>", map_fn(bake = "|l| l.dots.add_root().dots"))]
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

#[derive(Subcommand, Debug, Clone)]
#[cfg_attr(test, derive(Dummy, PartialEq, Eq))]
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
    link: LinkRaw,
  },

  /// Installs applications using the provided commands
  Install {
    #[clap(flatten)]
    install: InstallRaw,
  },

  #[cfg(not(test))]
  #[clap(verbatim_doc_comment)]
  /// Adds completions to shell
  ///
  /// Depending on your shell, you'll have to look up where to place the completion file.
  /// The recommended filename is _rotz
  ///
  /// - In bash, if you have `bash-completion` installed, you can place the completion file in `~/.local/share/bash-completion`
  /// - In zsh, place it anywhere inside a directory in your `$fpath` variable
  /// - In fish, place it in `~/.config/fish/completions`
  /// - In powershell, source the file in your `$PROFILE`
  Completions { shell: Option<Shell> },
}

impl Provider for Cli {
  fn metadata(&self) -> Metadata {
    Metadata::named("Cli")
  }

  #[cfg_attr(feature = "profiling", instrument)]
  fn data(&self) -> Result<Map<Profile, Dict>, Error> {
    let mut dict = Dict::new();

    if let Some(dotfiles) = &self.dotfiles {
      dict.insert("dotfiles".to_owned(), Value::serialize(dotfiles.to_string())?);
    }

    if let Command::Link {
      link: LinkRaw { link_type: Some(link_type), .. },
    } = &self.command
    {
      dict.insert("link_type".to_owned(), Value::serialize(link_type)?);
    }

    map! {
      Profile::Global => dict
    }
    .pipe(Ok)
  }
}
