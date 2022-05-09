#![cfg_attr(all(nightly, coverage), feature(no_coverage))]

use std::fs::{self, File};

use clap::Parser;
use directories::{ProjectDirs, UserDirs};
use figment::{
  providers::{Env, Format, Serialized},
  Figment,
};
use miette::{miette, Context, IntoDiagnostic, Result};
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
use cli::{Cli, Command};

mod config;
use config::Config;

mod dot;

mod commands;

pub static PROJECT_DIRS: Lazy<ProjectDirs> = Lazy::new(|| ProjectDirs::from("com", "", "rotz").ok_or_else(|| miette!("Could not get application data directory")).unwrap());
pub static USER_DIRS: Lazy<UserDirs> = Lazy::new(|| UserDirs::new().ok_or_else(|| miette!("Could not get user directory folder")).unwrap());

#[cfg(feature = "toml")]
pub const FILE_EXTENSION: &str = "toml";
#[cfg(feature = "yaml")]
pub const FILE_EXTENSION: &str = "yaml";
#[cfg(feature = "json")]
pub const FILE_EXTENSION: &str = "json";

fn main() -> Result<()> {
  let cli = Cli::parse();

  if !cli.config.0.exists() {
    fs::create_dir_all(cli.config.0.parent().ok_or_else(|| miette!("Could parse config file directory"))?)
      .into_diagnostic()
      .context("Could not create config file directory")?;
    File::create(&cli.config.0).into_diagnostic().context("Could not create default config file")?;
  }

  let mut config = Figment::from(Serialized::defaults(Config::default()));

  let config_str = fs::read_to_string(&cli.config.0).into_diagnostic().context("Could not read config file")?;
  if !cfg!(feature = "yaml") || !config_str.is_empty() {
    #[cfg(feature = "toml")]
    let config_file = Toml::string(&config_str);
    #[cfg(feature = "yaml")]
    let config_file = Yaml::string(&config_str);
    #[cfg(feature = "json")]
    let config_file = Json::string(&config_str);

    config = config.merge(config_file);
  }

  let mut config: Config = config.merge(Env::prefixed("ROTZ_")).merge(&cli).extract().into_diagnostic().context("Cloud not parse config")?;

  if config.dotfiles.starts_with("~/") {
    let mut iter = config.dotfiles.iter();
    iter.next();
    config.dotfiles = USER_DIRS.home_dir().iter().chain(iter).collect()
  }

  match cli.command {
    Command::Link { dots, link_type: _, force } => commands::link::execute(config, force, dots.dots),
    Command::Clone { repo: _ } => {
      if let Some(repo) = &config.repo {
        config::create_config_file_with_repo(repo, &cli.config.0)?;
      }
      commands::clone::execute(config)
    }
    _ => todo!(),
  }
}
