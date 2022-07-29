use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

use clap::ArgEnum;
use derive_more::{Display, IsVariant};
#[cfg(test)]
use fake::{Dummy, Fake};
use miette::{Diagnostic, NamedSource, Result, SourceSpan};
use serde::{Deserialize, Serialize};
use somok::Somok;

use crate::USER_DIRS;

#[derive(Debug, ArgEnum, Clone, Display, Deserialize, Serialize, IsVariant)]
#[cfg_attr(test, derive(Dummy, PartialEq))]
pub enum LinkType {
  /// Uses symbolic links for linking
  Symbolic,
  /// Uses hard links for linking
  Hard,
}

#[cfg(test)]
struct ValueFaker;

#[cfg(test)]
impl Dummy<ValueFaker> for HashMap<String, serde_json::Value> {
  fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &ValueFaker, rng: &mut R) -> Self {
    let mut map = HashMap::new();

    for _ in 0..10.fake_with_rng(rng) {
      map.insert((0..10).fake_with_rng(rng), serde_json::Value::String((0..10).fake_with_rng::<String, R>(rng)));
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

  /// The url of the repository passed to the git clone command
  pub(crate) repo: Option<String>,

  /// The command used to spawn processess.
  /// Use handlebars templates `{{ cmd }}` as placeholder for the cmd set in the dot.
  /// E.g. `"bash -c {{ quote "" cmd }}"`.
  pub(crate) shell_command: Option<String>,

  /// Variables can be used for templating in dot.(yaml|toml|json) files.
  #[cfg_attr(test, dummy(faker = "ValueFaker"))]
  pub(crate) variables: HashMap<String, serde_json::Value>,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      dotfiles: USER_DIRS.home_dir().join(".dotfiles"),
      link_type: LinkType::Symbolic,
      repo: None,
      #[cfg(windows)]
      shell_command: Some("pwsh -NoProfile -C {{ quote \"\" cmd }}".to_string()),
      #[cfg(all(not(target_os = "macos"), unix))]
      shell_command: Some("bash -c {{ quote \"\" cmd }}".to_string()),
      #[cfg(target_os = "macos")]
      shell_command: Some("zsh -c {{ quote \"\" cmd }}".to_string()),
      variables: HashMap::new(),
    }
  }
}

#[cfg(feature = "yaml")]
fn deserialize_config(config: &str) -> Result<Config, serde_yaml::Error> {
  serde_yaml::from_str(config)
}

#[cfg(feature = "yaml")]
fn serialize(config: &impl Serialize) -> Result<String, serde_yaml::Error> {
  serde_yaml::to_string(&config)
}

#[derive(thiserror::Error, Diagnostic, Debug)]
#[error("Config file already exists")]
#[diagnostic(code(config::write))]
pub struct AlreadyExistsError {
  name: String,
  #[label("{name} is already set")]
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
        (starts[0].0, pat.len()).into()
      } else {
        (0, content.len()).into()
      }
    };

    Self { name: name.to_string(), span }
  }
}

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum Error {
  #[cfg(feature = "yaml")]
  #[error("Could not serialize config")]
  #[diagnostic(code(config::serialize))]
  SerializingConfig(#[source] serde_yaml::Error),

  #[error("Could not write config")]
  #[diagnostic(code(config::write))]
  WritingConfig(PathBuf, #[source] std::io::Error),

  #[error("Config file already exists")]
  #[diagnostic(code(config::write))]
  AlreadyExists(#[source_code] NamedSource, #[related] Vec<AlreadyExistsError>),
}

#[cfg_attr(all(nightly, coverage), no_coverage)]
pub fn create_config_file(repo: Option<&str>, dotfiles: Option<&Path>, config_file: &Path) -> Result<(), Error> {
  if let Ok(existing_config_str) = fs::read_to_string(config_file) {
    if let Ok(existing_config) = deserialize_config(&existing_config_str) {
      let mut errors: Vec<AlreadyExistsError> = vec![];

      if let Some(repo) = repo {
        if existing_config.repo.as_ref().map_or(false, |r| *r != *repo) {
          errors.push(AlreadyExistsError::new("repo", &existing_config_str));
        }
      }

      if let Some(dotfiles) = dotfiles {
        if existing_config.dotfiles != dotfiles {
          errors.push(AlreadyExistsError::new("dotfiles", &existing_config_str));
        }
      }

      return Error::AlreadyExists(NamedSource::new(config_file.to_string_lossy(), existing_config_str), errors).error();
    }
  }

  let mut map = HashMap::new();

  if let Some(repo) = repo {
    map.insert("repo", repo.to_string());
  }

  if let Some(dotfiles) = dotfiles {
    map.insert("dotfiles", dotfiles.display().to_string());
  }

  fs::write(config_file, serialize(&map).map_err(Error::SerializingConfig)?).map_err(|e| Error::WritingConfig(config_file.to_path_buf(), e))?;

  ().okay()
}

#[cfg(test)]
mod tests {
  use fake::{Fake, Faker};
  use rstest::rstest;
  use speculoos::prelude::*;

  use super::Config;

  #[rstest]
  #[case(Faker.fake::<Config>())]
  #[case(Config::default())]
  fn ser_de(#[case] config: Config) {
    let serialized = super::serialize(&config);
    assert_that!(&serialized).is_ok();
    let serialized = serialized.unwrap();

    let deserialized = super::deserialize_config(&serialized);
    assert_that!(&deserialized).is_ok().is_equal_to(config);
  }
}
