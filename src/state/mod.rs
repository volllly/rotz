use std::{collections::HashMap, fs, path::PathBuf};

use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use tap::Pipe;

use crate::{helpers, FileFormat, FILE_EXTENSIONS, PROJECT_DIRS};

#[derive(thiserror::Error, Diagnostic, Debug)]
pub(crate) enum Error {
  #[error("Could not read state file")]
  #[diagnostic(code(state::read))]
  Reading(#[source] std::io::Error),

  #[error("Could not write state file")]
  #[diagnostic(code(state::write))]
  Writing(#[source] std::io::Error),

  #[error("Could not serialize state")]
  #[diagnostic(code(state::serialize))]
  Serializing(#[source] helpers::ParseError),

  #[error("Could not deserialize state")]
  #[diagnostic(code(state::deserialize))]
  Deserializing(#[source] helpers::ParseError),
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct State {
  pub linked: HashMap<String, HashMap<PathBuf, PathBuf>>,
}

impl State {
  pub fn read() -> Result<State, Error> {
    let state_file = helpers::get_file_with_format(PROJECT_DIRS.data_local_dir(), "state");

    if let Some((state_file, format)) = state_file {
      deserialize_state(&fs::read_to_string(state_file).map_err(Error::Reading)?, format).map_err(Error::Deserializing)?
    } else {
      State::default()
    }
    .pipe(Ok)
  }

  pub fn write(&self) -> Result<(), Error> {
    let state_file =
      helpers::get_file_with_format(PROJECT_DIRS.data_local_dir(), "state").unwrap_or_else(|| (PROJECT_DIRS.data_local_dir().join(format!("state.{}", FILE_EXTENSIONS[0].0)), FILE_EXTENSIONS[0].1));

    fs::create_dir_all(PROJECT_DIRS.data_local_dir()).map_err(Error::Writing)?;
    fs::write(state_file.0, serialize_state(self, state_file.1).map_err(Error::Serializing)?).map_err(Error::Writing)
  }
}

fn deserialize_state(state: &str, format: FileFormat) -> Result<State, helpers::ParseError> {
  Ok(match format {
    #[cfg(feature = "yaml")]
    FileFormat::Yaml => serde_yaml::from_str(state)?,
    #[cfg(feature = "toml")]
    FileFormat::Toml => serde_toml::from_str(state)?,
    #[cfg(feature = "json")]
    FileFormat::Json => serde_json::from_str(state)?,
  })
}

fn serialize_state(state: &impl Serialize, format: FileFormat) -> Result<String, helpers::ParseError> {
  Ok(match format {
    #[cfg(feature = "yaml")]
    FileFormat::Yaml => serde_yaml::to_string(state)?,
    #[cfg(feature = "toml")]
    FileFormat::Toml => serde_toml::to_string(state)?,
    #[cfg(feature = "json")]
    FileFormat::Json => serde_json::to_string(state)?,
  })
}
