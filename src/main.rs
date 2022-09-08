#![cfg_attr(all(nightly, coverage), feature(no_coverage))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::use_self)]
#![allow(clippy::default_trait_access)]
#![warn(clippy::filetype_is_file)]
#![warn(clippy::string_to_string)]
#![warn(clippy::unneeded_field_pattern)]
#![warn(clippy::mod_module_files)]
#![warn(clippy::str_to_string)]

use std::{
  convert::TryFrom,
  fs::{self, File},
  path::{Path, PathBuf},
};

use clap::Parser;
use commands::Command;
use derive_more::Display;
use directories::{ProjectDirs, UserDirs};
use figment::{
  providers::{Env, Format, Serialized},
  Figment,
};
use helpers::os;
use miette::{Diagnostic, Result, SourceSpan};
use once_cell::sync::Lazy;

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
use somok::Somok;

mod commands;
mod dot;
mod templating;

#[cfg(not(any(feature = "toml", feature = "yaml", feature = "json")))]
compile_error!("At least one file format features needs to be enabled");

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("Unknown file extension")]
  #[diagnostic(code(parse::extension))]
  UnknownExtension(#[source_code] String, #[label] SourceSpan),

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

pub(crate) static PROJECT_DIRS: Lazy<ProjectDirs> = Lazy::new(|| ProjectDirs::from("com", "", "rotz").ok_or(Error::GettingDirs("application data")).expect("Could not read project dirs"));
pub(crate) static USER_DIRS: Lazy<UserDirs> = Lazy::new(|| UserDirs::new().ok_or(Error::GettingDirs("user")).expect("Could not read user dirs"));
pub(crate) const FILE_EXTENSIONS_GLOB: &str = "{y<a>ml,toml,json}";
pub(crate) const FILE_EXTENSIONS: &[(&str, FileFormat)] = &[
  #[cfg(feature = "yaml")]
  ("yaml", FileFormat::Yaml),
  #[cfg(feature = "yaml")]
  ("yml", FileFormat::Yaml),
  #[cfg(feature = "toml")]
  ("toml", FileFormat::Toml),
  #[cfg(feature = "json")]
  ("json", FileFormat::Json),
];

#[derive(Debug, Display, Clone, Copy)]
pub(crate) enum FileFormat {
  #[cfg(feature = "yaml")]
  Yaml,
  #[cfg(feature = "toml")]
  Toml,
  #[cfg(feature = "json")]
  Json,
}

impl TryFrom<&str> for FileFormat {
  type Error = Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    FILE_EXTENSIONS
      .iter()
      .find(|e| e.0 == value)
      .map(|e| e.1)
      .ok_or_else(|| Error::UnknownExtension(value.to_owned(), (0, value.len()).into()))
  }
}

impl TryFrom<&Path> for FileFormat {
  type Error = Error;

  fn try_from(value: &Path) -> Result<Self, Self::Error> {
    value.extension().map_or_else(
      || Error::UnknownExtension(value.to_string_lossy().to_string(), (0, 0).into()).error(),
      |extension| {
        FILE_EXTENSIONS
          .iter()
          .find(|e| e.0 == extension)
          .map(|e| e.1)
          .ok_or_else(|| Error::UnknownExtension(extension.to_string_lossy().to_string(), (0, extension.len()).into()))
      },
    )
  }
}

fn main() -> Result<(), miette::Report> {
  let cli = Cli::parse();

  if !cli.config.0.exists() {
    fs::create_dir_all(cli.config.0.parent().ok_or_else(|| Error::ParsingConfigDir(cli.config.0.clone()))?).map_err(|e| Error::CreatingConfig(cli.config.0.clone(), e))?;
    File::create(&cli.config.0).map_err(|e| Error::CreatingConfig(cli.config.0.clone(), e))?;
  }

  let mut config_figment = Figment::from(Serialized::defaults(Config::default()));

  config_figment = merge_global_config(&cli.config.0, config_figment)?.merge(Env::prefixed("ROTZ_")).merge(&cli);

  let mut config: Config = config_figment.extract().map_err(Error::ParsingConfig)?;

  if config.dotfiles.starts_with("~/") {
    let mut iter = config.dotfiles.iter();
    iter.next();
    config.dotfiles = USER_DIRS.home_dir().iter().chain(iter).collect();
  }

  config_figment = join_repo_config(&config.dotfiles, config_figment)?;

  config = config_figment
    .clone()
    .select("global")
    .merge(config_figment.select(os::OS.to_string().to_ascii_lowercase()))
    .extract()
    .map_err(Error::ParsingConfig)?;

  match cli.command.clone() {
    cli::Command::Link { link } => commands::Link::new(config).execute((cli.bake(), link.bake())),
    cli::Command::Clone { repo } => commands::Clone::new(config).execute((cli, repo)),
    cli::Command::Install { install } => commands::Install::new(config).execute((cli.bake(), install.bake())),
    cli::Command::Sync { sync } => commands::Sync::new(config).execute((cli.bake(), sync.bake())),
    cli::Command::Init { repo } => commands::Init::new(config).execute((cli, repo)),
  }
}

fn merge_global_config(path: &Path, mut config: Figment) -> Result<Figment, Error> {
  let config_str = fs::read_to_string(path).map_err(|e| Error::ReadingConfig(path.to_path_buf(), e))?;
  if !config_str.is_empty() {
    let file_extension = &*path.extension().unwrap().to_string_lossy();
    config = match file_extension {
      #[cfg(feature = "yaml")]
      "yaml" | "yml" => config.merge(Yaml::string(&config_str)),
      #[cfg(feature = "toml")]
      "toml" => config.merge(Toml::string(&config_str)),
      #[cfg(feature = "json")]
      "json" => config.merge(Json::string(&config_str)),
      _ => {
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        return Error::UnknownExtension(file_name.clone(), (file_name.rfind(file_extension).unwrap(), file_extension.len()).into()).error();
      }
    };
  }

  config.okay()
}

fn join_repo_config(path: &Path, mut config: Figment) -> Result<Figment, Error> {
  if let Some((path, format)) = helpers::get_file_with_format(path, "config") {
    let config_str = fs::read_to_string(&path).map_err(|e| Error::ReadingConfig(path.clone(), e))?;
    if !config_str.is_empty() {
      config = match format {
        #[cfg(feature = "yaml")]
        FileFormat::Yaml => config.join(Yaml::string(&config_str)),
        #[cfg(feature = "toml")]
        FileFormat::Toml => config.join(Toml::string(&config_str)),
        #[cfg(feature = "json")]
        FileFormat::Json => config.join(Json::string(&config_str)),
      };
    }
  }
  config.okay()
}
