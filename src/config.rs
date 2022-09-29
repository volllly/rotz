use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

use clap::ValueEnum;
use crossterm::style::Stylize;
use derive_more::{Display, IsVariant};
#[cfg(test)]
use fake::{Dummy, Fake};
use figment::{providers::Serialized, value, Metadata, Profile, Provider};
use miette::{Diagnostic, NamedSource, Result, SourceSpan};
use path_absolutize::Absolutize;
use serde::{Deserialize, Serialize};
use tap::{Pipe, TryConv};

use crate::{helpers, FileFormat, USER_DIRS};

#[derive(Debug, ValueEnum, Clone, Display, Deserialize, Serialize, IsVariant)]
#[cfg_attr(test, derive(Dummy, PartialEq, Eq))]
pub enum LinkType {
  /// Uses symbolic links for linking
  Symbolic,
  /// Uses hard links for linking
  Hard,
}

#[cfg(test)]
struct ValueFaker;

#[cfg(test)]
#[allow(clippy::implicit_hasher)]
impl Dummy<ValueFaker> for figment::value::Dict {
  fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &ValueFaker, rng: &mut R) -> Self {
    let mut map = Self::new();

    for _ in 0..((0..10).fake_with_rng(rng)) {
      map.insert((0..10).fake_with_rng(rng), (0..10).fake_with_rng::<String, R>(rng).into());
    }

    map
  }
}

#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(test, derive(Dummy, PartialEq))]
pub struct Config {
  /// Path to the local dotfiles
  pub(crate) dotfiles: PathBuf,

  /// Which link type to use for linking dotfiles
  pub(crate) link_type: LinkType,

  /// The command used to spawn processess.
  /// Use handlebars templates `{{ cmd }}` as placeholder for the cmd set in the dot.
  /// E.g. `"bash -c {{ quote "" cmd }}"`.
  pub(crate) shell_command: Option<String>,

  /// Variables can be used for templating in dot.(yaml|toml|json) files.
  #[cfg_attr(test, dummy(faker = "ValueFaker"))]
  pub(crate) variables: figment::value::Dict,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      dotfiles: USER_DIRS.home_dir().join(".dotfiles"),
      link_type: LinkType::Symbolic,
      #[cfg(windows)]
      shell_command: Some("powershell -NoProfile -C {{ quote \"\" cmd }}".to_owned()),
      #[cfg(all(not(target_os = "macos"), unix))]
      shell_command: Some("bash -c {{ quote \"\" cmd }}".to_owned()),
      #[cfg(target_os = "macos")]
      shell_command: Some("zsh -c {{ quote \"\" cmd }}".to_owned()),
      variables: figment::value::Dict::new(),
    }
  }
}

impl Provider for Config {
  fn metadata(&self) -> Metadata {
    Metadata::named("Library Config")
  }

  fn data(&self) -> Result<value::Map<Profile, value::Dict>, figment::Error> {
    Serialized::defaults(Config::default()).data()
  }

  fn profile(&self) -> Option<Profile> {
    None
  }
}

fn deserialize_config(config: &str, format: FileFormat) -> Result<Config, helpers::ParseError> {
  Ok(match format {
    #[cfg(feature = "yaml")]
    FileFormat::Yaml => serde_yaml::from_str(config)?,
    #[cfg(feature = "toml")]
    FileFormat::Toml => serde_toml::from_str(config)?,
    #[cfg(feature = "json")]
    FileFormat::Json => serde_json::from_str(config)?,
  })
}

fn serialize_config(config: &impl Serialize, format: FileFormat) -> Result<String, helpers::ParseError> {
  Ok(match format {
    #[cfg(feature = "yaml")]
    FileFormat::Yaml => serde_yaml::to_string(config)?,
    #[cfg(feature = "toml")]
    FileFormat::Toml => serde_toml::to_string(config)?,
    #[cfg(feature = "json")]
    FileFormat::Json => serde_json::to_string(config)?,
  })
}

#[derive(thiserror::Error, Diagnostic, Debug)]
#[error("{name} is already set")]
#[diagnostic(code(config::exists::value))]
pub struct AlreadyExistsError {
  name: String,
  #[label("{name} is set here")]
  span: SourceSpan,
}

impl AlreadyExistsError {
  pub fn new(name: &str, content: &str) -> Self {
    let pat = format!("{name}: ");
    let span: SourceSpan = if content.starts_with(&pat) {
      (0, pat.len()).into()
    } else {
      let starts = content.match_indices(&format!("\n{pat}")).collect::<Vec<_>>();
      if starts.len() == 1 {
        (starts[0].0 + 1, pat.len()).into()
      } else {
        (0, content.len()).into()
      }
    };

    Self { name: name.to_owned(), span }
  }
}

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum Error {
  #[cfg(feature = "yaml")]
  #[error("Could not serialize config")]
  #[diagnostic(code(config::serialize))]
  SerializingConfig(#[source] helpers::ParseError),

  #[error("Could not write config")]
  #[diagnostic(code(config::write))]
  WritingConfig(PathBuf, #[source] std::io::Error),

  #[error("Could not get absolute path")]
  #[diagnostic(code(config::canonicalize))]
  Canonicalize(#[source] std::io::Error),

  #[error("Config file already exists")]
  #[diagnostic(code(config::exists))]
  AlreadyExists(#[label] Option<SourceSpan>, #[source_code] NamedSource, #[related] Vec<AlreadyExistsError>),

  #[error("Could not parse dotfiles directory \"{0}\"")]
  #[diagnostic(code(config::filename::parse), help("Did you enter a valid file?"))]
  PathParse(PathBuf),

  #[error(transparent)]
  InvalidFileFormat(#[from] crate::Error),
}

#[cfg_attr(all(nightly, coverage), no_coverage)]
pub fn create_config_file(dotfiles: Option<&Path>, config_file: &Path) -> Result<(), Error> {
  let format = config_file.try_conv::<FileFormat>()?;

  if let Ok(existing_config_str) = fs::read_to_string(config_file) {
    if let Ok(existing_config) = deserialize_config(&existing_config_str, format) {
      let mut errors: Vec<AlreadyExistsError> = vec![];

      if let Some(dotfiles) = dotfiles {
        if existing_config.dotfiles != dotfiles {
          errors.push(AlreadyExistsError::new("dotfiles", &existing_config_str));
        }
      }

      return Error::AlreadyExists(
        errors.is_empty().then(|| (0, existing_config_str.len()).into()),
        NamedSource::new(config_file.to_string_lossy(), existing_config_str),
        errors,
      )
      .pipe(Err);
    }
  }

  let mut map = HashMap::new();

  if let Some(dotfiles) = dotfiles {
    map.insert(
      "dotfiles",
      dotfiles
        .absolutize()
        .map_err(Error::Canonicalize)?
        .to_str()
        .ok_or_else(|| Error::PathParse(dotfiles.to_path_buf()))?
        .to_owned(),
    );
  }

  fs::write(config_file, serialize_config(&map, format).map_err(Error::SerializingConfig)?).map_err(|e| Error::WritingConfig(config_file.to_path_buf(), e))?;

  println!("Created config file at {}", config_file.to_string_lossy().green());

  ().pipe(Ok)
}

pub struct MappedProfileProvider<P: Provider> {
  pub mapping: HashMap<Profile, Profile>,
  pub provider: P,
}

impl<P: Provider> Provider for MappedProfileProvider<P> {
  fn metadata(&self) -> Metadata {
    self.provider.metadata()
  }

  fn data(&self) -> Result<value::Map<Profile, value::Dict>, figment::Error> {
    let data = self.provider.data()?;
    let mut mapped = value::Map::<Profile, value::Dict>::new();

    for (profile, data) in data {
      mapped.insert(self.mapping.get(&profile).map_or(profile, Clone::clone), data);
    }

    mapped.pipe(Ok)
  }
}

#[cfg(test)]
mod tests {
  use fake::{Fake, Faker};
  use rstest::rstest;
  use speculoos::prelude::*;

  use super::Config;
  use crate::FileFormat;

  #[rstest]
  fn ser_de(#[values(Faker.fake::<Config>(), Config::default())] config: Config, #[values(FileFormat::Yaml, FileFormat::Toml, FileFormat::Json)] format: FileFormat) {
    let serialized = super::serialize_config(&config, format);
    let serialized = assert_that!(&serialized).is_ok().subject;

    let deserialized = super::deserialize_config(serialized, format);
    assert_that!(&deserialized).is_ok().is_equal_to(config);
  }
}
