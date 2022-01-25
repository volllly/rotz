use std::fs::{self, File};

use clap::Parser;
use directories::ProjectDirs;
use figment::{
  providers::{Env, Format, Serialized},
  Figment,
};
use once_cell::sync::Lazy;

#[cfg(any(all(feature = "toml", feature = "yaml"), all(feature = "toml", feature = "json"), all(feature = "json", feature = "yaml")))]
compile_error!("multiple file format features may not be used at the same time");

#[cfg(feature = "json")]
use figment::providers::Json;
#[cfg(feature = "toml")]
use figment::providers::Toml;
#[cfg(feature = "yaml")]
use figment::providers::Yaml;

mod cli;
use cli::{Cli, Command};

mod config;
use config::Config;

mod dot;

mod commands;

pub static PROJECT_DIRS: Lazy<ProjectDirs> = Lazy::new(|| ProjectDirs::from("com", "volavsek", "rotz").unwrap());

#[cfg(feature = "toml")]
pub const FILE_EXTENSION: &str = "toml";
#[cfg(feature = "yaml")]
pub const FILE_EXTENSION: &str = "yaml";
#[cfg(feature = "json")]
pub const FILE_EXTENSION: &str = "json";

fn main() {
  let cli = Cli::parse();

  if !cli.config.0.exists() {
    fs::create_dir_all(cli.config.0.parent().unwrap()).unwrap();
    File::create(&cli.config.0).unwrap();
  }

  #[cfg(feature = "toml")]
  let config_file = Toml::file(&cli.config.0);
  #[cfg(feature = "yaml")]
  let config_file = Yaml::file(&cli.config.0);
  #[cfg(feature = "json")]
  let config_file = Json::file(&cli.config.0);

  let mut config = Figment::from(Serialized::defaults(Config::default()));

  if cfg!(feature = "yaml") && !fs::read_to_string(&cli.config.0).unwrap().is_empty() {
    config = config.merge(config_file);
  }

  let config: Config = config.merge(Env::prefixed("ROTZ_")).merge(&cli).extract().unwrap();

  match cli.command {
    Command::Link { dots, link_type: _, force } => commands::link::execute(config, force, dots.dots),
    _ => todo!(),
  }
}
