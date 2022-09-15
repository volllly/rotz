#![cfg_attr(all(nightly, coverage), feature(no_coverage))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::use_self)]
#![allow(clippy::default_trait_access)]
#![warn(clippy::filetype_is_file)]
#![warn(clippy::string_to_string)]
#![warn(clippy::unneeded_field_pattern)]
#![warn(clippy::self_named_module_files)]
#![warn(clippy::str_to_string)]

use std::{
  collections::HashMap,
  convert::TryFrom,
  fs::{self, File},
  path::{Path, PathBuf},
};

use clap::Parser;
use commands::Command;
use derive_more::Display;
use directories::{ProjectDirs, UserDirs};
#[cfg(feature = "json")]
use figment::providers::Json;
#[cfg(feature = "toml")]
use figment::providers::Toml;
#[cfg(feature = "yaml")]
use figment::providers::Yaml;
use figment::{
  providers::{Env, Format},
  Figment, Profile,
};
use helpers::os;
use miette::{Diagnostic, Result, SourceSpan};
use once_cell::sync::Lazy;

mod helpers;

mod cli;
use cli::Cli;

mod config;
use config::{Config, MappedProfileProvider};
use tap::Pipe;
use templating::init_handlebars;
use velcro::hash_map;

mod commands;
mod dot;
mod templating;

#[cfg(not(any(feature = "toml", feature = "yaml", feature = "json")))]
compile_error!("At least one file format features needs to be enabled");

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum Error {
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

  #[error("Cloud not parse config")]
  #[diagnostic(code(config::parse), help("Is the config file in the correct format?"))]
  ParsingConfig(#[source] figment::Error),

  #[error("Cloud not parse config")]
  #[diagnostic(code(config::local::parse), help("Did you provide a top level \"global\" key in the repo level config?"))]
  RepoConfigProfile(#[source] figment::Error),
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
  #[display(fmt = "yaml")]
  Yaml,
  #[cfg(feature = "toml")]
  #[display(fmt = "toml")]
  Toml,
  #[cfg(feature = "json")]
  #[display(fmt = "json")]
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
      || Error::UnknownExtension(value.to_string_lossy().to_string(), (0, 0).into()).pipe(Err),
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

  let config = read_config(&cli)?;

  init_handlebars(&config, &cli)?;

  match cli.command.clone() {
    cli::Command::Link { link } => commands::Link::new(config).execute((cli.bake(), link.bake())),
    cli::Command::Clone { repo } => commands::Clone::new(config).execute((cli, repo)),
    cli::Command::Install { install } => commands::Install::new(config).execute((cli.bake(), install.bake())),
    cli::Command::Sync { sync } => commands::Sync::new(config).execute((cli.bake(), sync.bake())),
    cli::Command::Init { repo } => commands::Init::new(config).execute((cli, repo)),
  }
}

fn read_config(cli: &Cli) -> Result<Config, Error> {
  let env_config = Env::prefixed("ROTZ_");

  let mut figment = Figment::new().merge_from_path(&cli.config.0, false)?.merge(env_config).merge(&cli);

  let config: Config = figment.clone().join(Config::default()).extract().map_err(Error::ParsingConfig)?;

  let dotfiles = if config.dotfiles.starts_with("~/") {
    let mut iter = config.dotfiles.iter();
    iter.next();
    USER_DIRS.home_dir().iter().chain(iter).collect()
  } else {
    config.dotfiles
  };

  if let Some((config, _)) = helpers::get_file_with_format(dotfiles, "config") {
    figment = figment.join_from_path(config, true, hash_map!( "global".into(): "default".into(), "force".into(): "global".into() ))?;
  }

  figment
    .join(Config::default())
    .select(os::OS.to_string().to_ascii_lowercase())
    .extract()
    .map_err(Error::RepoConfigProfile)
}

trait FigmentExt {
  fn merge_from_path(self, path: impl AsRef<Path>, nested: bool) -> Result<Self, Error>
  where
    Self: std::marker::Sized;
  fn join_from_path(self, path: impl AsRef<Path>, nested: bool, mapping: HashMap<Profile, Profile>) -> Result<Self, Error>
  where
    Self: std::marker::Sized;
}

trait DataExt {
  fn set_nested(self, nested: bool) -> Self
  where
    Self: std::marker::Sized;
}

impl<F: Format> DataExt for figment::providers::Data<F> {
  fn set_nested(self, nested: bool) -> Self
  where
    Self: std::marker::Sized,
  {
    if nested {
      self.nested()
    } else {
      self
    }
  }
}

impl FigmentExt for Figment {
  fn merge_from_path(self, path: impl AsRef<Path>, nested: bool) -> Result<Self, Error> {
    let config_str = fs::read_to_string(&path).map_err(|e| Error::ReadingConfig(path.as_ref().to_path_buf(), e))?;
    if !config_str.is_empty() {
      let file_extension = &*path.as_ref().extension().unwrap().to_string_lossy();
      return match file_extension {
        #[cfg(feature = "yaml")]
        "yaml" | "yml" => self.merge(Yaml::string(&config_str).set_nested(nested)),
        #[cfg(feature = "toml")]
        "toml" => self.merge(Toml::string(&config_str).set_nested(nested)),
        #[cfg(feature = "json")]
        "json" => self.merge(Json::string(&config_str).set_nested(nested)),
        _ => {
          let file_name = path.as_ref().file_name().unwrap().to_string_lossy().to_string();
          return Error::UnknownExtension(file_name.clone(), (file_name.rfind(file_extension).unwrap(), file_extension.len()).into()).pipe(Err);
        }
      }
      .pipe(Ok);
    }

    self.pipe(Ok)
  }

  fn join_from_path(self, path: impl AsRef<Path>, nested: bool, mapping: HashMap<Profile, Profile>) -> Result<Self, Error>
  where
    Self: std::marker::Sized,
  {
    let config_str = fs::read_to_string(&path).map_err(|e| Error::ReadingConfig(path.as_ref().to_path_buf(), e))?;
    if !config_str.is_empty() {
      let file_extension = &*path.as_ref().extension().unwrap().to_string_lossy();
      return match file_extension {
        #[cfg(feature = "yaml")]
        "yaml" | "yml" => self.join(MappedProfileProvider {
          mapping,
          provider: Yaml::string(&config_str).set_nested(nested),
        }),
        #[cfg(feature = "toml")]
        "toml" => self.join(MappedProfileProvider {
          mapping,
          provider: Toml::string(&config_str).set_nested(nested),
        }),
        #[cfg(feature = "json")]
        "json" => self.join(MappedProfileProvider {
          mapping,
          provider: Json::string(&config_str).set_nested(nested),
        }),
        _ => {
          let file_name = path.as_ref().file_name().unwrap().to_string_lossy().to_string();
          return Error::UnknownExtension(file_name.clone(), (file_name.rfind(file_extension).unwrap(), file_extension.len()).into()).pipe(Err);
        }
      }
      .pipe(Ok);
    }

    self.pipe(Ok)
  }
}

#[cfg(test)]
mod test;
