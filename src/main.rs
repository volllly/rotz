use std::{
  collections::{HashMap, HashSet},
  convert::TryFrom,
  fs::{self, File},
  path::{Path, PathBuf},
};

use clap::Parser;
use commands::Command;
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
use strum::Display;

mod helpers;

mod cli;
use cli::Cli;

mod config;
use config::{Config, MappedProfileProvider};
use state::State;
use tap::Pipe;
#[cfg(feature = "profiling")]
use tracing::instrument;
use velcro::hash_map;

mod commands;
mod dot;
mod state;
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

  #[error("Default profile not allowed in config")]
  #[diagnostic(code(config::local::default), help("Change the top level \"default\" key in the repo level config to \"global\""))]
  RepoConfigDefaultProfile,
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
  #[strum(to_string = "yaml")]
  Yaml,
  #[cfg(feature = "toml")]
  #[strum(to_string = "toml")]
  Toml,
  #[cfg(feature = "json")]
  #[strum(to_string = "json")]
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

#[cfg(feature = "profiling")]
fn main() -> Result<(), miette::Report> {
  use tracing_subscriber::prelude::*;
  use tracing_tracy::TracyLayer;

  let tracy_layer = TracyLayer::default();
  tracing_subscriber::registry().with(tracy_layer).init();

  let result = run();

  std::thread::sleep(std::time::Duration::from_secs(2));

  result
}

#[cfg(not(feature = "profiling"))]
fn main() -> Result<(), miette::Report> {
  run()
}

#[cfg_attr(feature = "profiling", instrument)]
fn run() -> Result<(), miette::Report> {
  let cli = Cli::parse();

  if !cli.config.0.exists() {
    fs::create_dir_all(cli.config.0.parent().ok_or_else(|| Error::ParsingConfigDir(cli.config.0.clone()))?).map_err(|e| Error::CreatingConfig(cli.config.0.clone(), e))?;
    File::create(&cli.config.0).map_err(|e| Error::CreatingConfig(cli.config.0.clone(), e))?;
  }

  let config = read_config(&cli)?;

  let engine = templating::Engine::new(&config, &cli);
  let mut state = State::read()?;
  match cli.command.clone() {
    cli::Command::Link { link } => commands::Link::new(config, engine)
      .execute((cli.bake(), link.bake(), &state.linked))
      .map(|linked| state.linked = linked),
    cli::Command::Clone { repo } => commands::Clone::new(config).execute((cli, repo)),
    cli::Command::Install { install } => commands::Install::new(config, engine).execute((cli.bake(), install.bake())),
    cli::Command::Init { repo } => commands::Init::new(config).execute((cli, repo)),
  }?;

  state.write().map_err(Into::into)
}

fn read_config(cli: &Cli) -> Result<Config, Error> {
  let env_config = Env::prefixed("ROTZ_");

  let mut figment = Figment::new().merge_from_path(&cli.config.0, false)?.merge(env_config).merge(cli);

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

  let tmp = figment.join(Config::default()).select(os::OS.to_string().to_ascii_lowercase());

  tmp.extract().map_err(Error::RepoConfigProfile)
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

#[derive(strum::Display, strum::EnumString)]
#[strum(ascii_case_insensitive)]
enum Profiles {
  Force,
  Global,
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

  fn join_from_path(self, path: impl AsRef<Path>, mut nested: bool, mapping: HashMap<Profile, Profile>) -> Result<Self, Error>
  where
    Self: std::marker::Sized,
  {
    let config_str = fs::read_to_string(&path).map_err(|e| Error::ReadingConfig(path.as_ref().to_path_buf(), e))?;
    if !config_str.is_empty() {
      let file_extension = &*path.as_ref().extension().unwrap().to_string_lossy();

      if nested {
        let profiles = match file_extension {
          #[cfg(feature = "yaml")]
          "yaml" | "yml" => serde_yaml::from_str::<serde_json::Map<String, serde_json::Value>>(&config_str).unwrap().pipe(Ok),
          #[cfg(feature = "toml")]
          "toml" => serde_toml::from_str::<serde_json::Map<String, serde_json::Value>>(&config_str).unwrap().pipe(Ok),
          #[cfg(feature = "json")]
          "json" => serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&config_str).unwrap().pipe(Ok),
          _ => {
            let file_name = path.as_ref().file_name().unwrap().to_string_lossy().to_string();
            return Error::UnknownExtension(file_name.clone(), (file_name.rfind(file_extension).unwrap(), file_extension.len()).into()).pipe(Err);
          }
        }?
        .into_iter()
        .map(|(k, _)| k)
        .collect::<HashSet<String>>();

        if profiles.contains(&Profile::Default.to_string().to_lowercase()) {
          Error::RepoConfigDefaultProfile.pipe(Err)?;
        }

        nested = profiles.iter().any(|p| Profiles::try_from(p.as_str()).is_ok() || os::Os::try_from(p.as_str()).is_ok());
      }

      match file_extension {
        #[cfg(feature = "yaml")]
        "yaml" | "yml" => {
          return self
            .join(MappedProfileProvider {
              mapping,
              provider: Yaml::string(&config_str).set_nested(nested),
            })
            .pipe(Ok)
        }
        #[cfg(feature = "toml")]
        "toml" => {
          return self
            .join(MappedProfileProvider {
              mapping,
              provider: Toml::string(&config_str).set_nested(nested),
            })
            .pipe(Ok)
        }
        #[cfg(feature = "json")]
        "json" => {
          return self
            .join(MappedProfileProvider {
              mapping,
              provider: Json::string(&config_str).set_nested(nested),
            })
            .pipe(Ok)
        }
        _ => {
          let file_name = path.as_ref().file_name().unwrap().to_string_lossy().to_string();
          return Error::UnknownExtension(file_name.clone(), (file_name.rfind(file_extension).unwrap(), file_extension.len()).into()).pipe(Err);
        }
      }
    }

    self.pipe(Ok)
  }
}

#[cfg(test)]
mod test;
