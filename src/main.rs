#![cfg_attr(all(nightly, coverage), feature(no_coverage))]

use std::{
  fs::{self, File},
  path::PathBuf,
};

use clap::Parser;
use commands::Command;
use directories::{ProjectDirs, UserDirs};
use figment::{
  providers::{Env, Format, Serialized},
  Figment,
};
use miette::{Diagnostic, Result};
use once_cell::sync::Lazy;

#[cfg(any(all(feature = "toml", feature = "yaml"), all(feature = "toml", feature = "json"), all(feature = "json", feature = "yaml")))]
compile_error!("multiple file format features may not be used at the same time");

#[cfg(feature = "json")]
use figment::providers::Json;
#[cfg(feature = "toml")]
use figment::providers::Toml;
#[cfg(feature = "yaml")]
use figment::providers::Yaml;

mod helpers;

mod cli;
use cli::Cli;

mod config;
use config::Config;

mod commands;
mod dot;

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("Could not get \"{0}\" directory")]
  #[diagnostic(code(project_dirs::not_found))]
  GettingDirs(&'static str),

  #[error("Could parse config file directory \"{0}\"")]
  #[diagnostic(code(config::parent_dir))]
  ParsingConfigDir(PathBuf),

  #[error("Could not create config file directory \"{0}\"")]
  #[diagnostic(code(config::create))]
  CreatingConfig(PathBuf, #[source] std::io::Error),

  #[error("Could not read config file \"{0}\"")]
  #[diagnostic(code(config::read), help("Do you have access to the config file?"))]
  ReadingConfig(PathBuf, #[source] std::io::Error),

  #[error("Cloud not parse config \"{0}\"")]
  #[diagnostic(code(config::parse), help("Is the config in the correct format?"))]
  ParsingConfig(#[source] figment::Error),
}

pub(crate) static PROJECT_DIRS: Lazy<ProjectDirs> = Lazy::new(|| ProjectDirs::from("com", "", "rotz").ok_or(Error::GettingDirs("application data")).unwrap());
pub(crate) static USER_DIRS: Lazy<UserDirs> = Lazy::new(|| UserDirs::new().ok_or(Error::GettingDirs("user")).unwrap());

#[cfg(feature = "toml")]
pub(crate) const FILE_EXTENSION: &str = "toml";
#[cfg(feature = "yaml")]
pub(crate) const FILE_EXTENSION: &str = "yaml";
#[cfg(feature = "json")]
pub(crate) const FILE_EXTENSION: &str = "json";

fn main() -> Result<()> {
  let cli = Cli::parse();

  if !cli.config.0.exists() {
    fs::create_dir_all(cli.config.0.parent().ok_or_else(|| Error::ParsingConfigDir(cli.config.0.clone()))?).map_err(|e| Error::CreatingConfig(cli.config.0.clone(), e))?;
    File::create(&cli.config.0).map_err(|e| Error::CreatingConfig(cli.config.0.clone(), e))?;
  }

  let mut config = Figment::from(Serialized::defaults(Config::default()));

  let config_str = fs::read_to_string(&cli.config.0).map_err(|e| Error::ReadingConfig(cli.config.0.clone(), e))?;
  if !cfg!(feature = "yaml") || !config_str.is_empty() {
    #[cfg(feature = "toml")]
    let config_file = Toml::string(&config_str);
    #[cfg(feature = "yaml")]
    let config_file = Yaml::string(&config_str);
    #[cfg(feature = "json")]
    let config_file = Json::string(&config_str);

    config = config.merge(config_file);
  }

  let mut config: Config = config.merge(Env::prefixed("ROTZ_")).merge(&cli).extract().map_err(Error::ParsingConfig)?;

  if config.dotfiles.starts_with("~/") {
    let mut iter = config.dotfiles.iter();
    iter.next();
    config.dotfiles = USER_DIRS.home_dir().iter().chain(iter).collect();
  }

  match cli.command {
    cli::Command::Link { link } => commands::Link::new(config).execute(link.bake()),
    cli::Command::Clone { repo: _ } => {
      if let Some(repo) = &config.repo {
        config::create_config_file_with_repo(repo, &cli.config.0)?;
      }
      commands::Clone::new(config).execute(())
    }
    cli::Command::Install { install } => commands::Install::new(config).execute(install.bake()),
    cli::Command::Sync { .. } => todo!(),
  }
}
