use std::{
  collections::HashMap,
  fmt::Debug,
  fs,
  path::{Path, PathBuf},
};

use clap::ValueEnum;
use crossterm::style::Stylize;
#[cfg(test)]
use fake::{Dummy, Fake};
use figment::{Metadata, Profile, Provider, providers::Serialized, value};
use miette::{Diagnostic, NamedSource, Result, SourceSpan};
use path_absolutize::Absolutize;
use serde::{Deserialize, Deserializer, Serialize};
use strum::{Display, EnumIs};
use tap::{Pipe, TryConv};
#[cfg(feature = "profiling")]
use tracing::instrument;

use crate::{FileFormat, USER_DIRS, helpers};

#[derive(Debug, ValueEnum, Clone, Display, Deserialize, Serialize, EnumIs)]
#[cfg_attr(test, derive(Dummy, PartialEq, Eq))]
pub enum LinkType {
  /// Uses symbolic links for linking
  Symbolic,
  /// Uses hard links for linking
  Hard,
}

/// Represents one or more dotfiles directories
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
#[cfg_attr(test, derive(Dummy, PartialEq))]
pub enum DotfilesPath {
  /// Single dotfiles directory (backward compatibility)
  Single(PathBuf),
  /// Multiple dotfiles directories (new functionality)
  Multiple(Vec<PathBuf>),
}

impl DotfilesPath {
  /// Get all paths as a slice
  pub fn paths(&self) -> &[PathBuf] {
    match self {
      DotfilesPath::Single(path) => std::slice::from_ref(path),
      DotfilesPath::Multiple(paths) => paths.as_slice(),
    }
  }

  /// Get the first (primary) path
  pub fn first_path(&self) -> &PathBuf {
    match self {
      DotfilesPath::Single(path) => path,
      DotfilesPath::Multiple(paths) => &paths[0],
    }
  }

  /// Iterate over all paths
  pub fn iter(&self) -> impl Iterator<Item = &PathBuf> {
    self.paths().iter()
  }

  /// Check if a specific path is contained
  pub fn contains_path(&self, path: &Path) -> bool {
    self.paths().iter().any(|p| p == path)
  }

  /// Get as single path (for backward compatibility)
  pub fn as_single(&self) -> Option<&PathBuf> {
    match self {
      DotfilesPath::Single(path) => Some(path),
      DotfilesPath::Multiple(_) => None,
    }
  }

  /// Check if this represents multiple paths
  pub fn is_multiple(&self) -> bool {
    matches!(self, DotfilesPath::Multiple(_))
  }

  /// Get the number of paths
  pub fn len(&self) -> usize {
    self.paths().len()
  }

  /// Check if empty (should not happen in valid configs)
  pub fn is_empty(&self) -> bool {
    self.paths().is_empty()
  }

  /// Get string representation for display (uses first path for multiple)
  pub fn to_string_lossy(&self) -> std::borrow::Cow<str> {
    self.first_path().to_string_lossy()
  }

  /// Get OsStr representation (uses first path for multiple)
  pub fn as_os_str(&self) -> &std::ffi::OsStr {
    self.first_path().as_os_str()
  }

  /// Join a path to the first dotfiles directory (for backward compatibility)
  pub fn join<P: AsRef<std::path::Path>>(&self, path: P) -> PathBuf {
    self.first_path().join(path)
  }

  /// Validate paths (no duplicates, all exist or can be created)
  pub fn validate(&self) -> std::result::Result<(), DotfilesPathError> {
    let paths = self.paths();

    // Check for empty
    if paths.is_empty() {
      return Err(DotfilesPathError::EmptyPaths);
    }

    // Check for duplicates after resolution
    let mut resolved_paths = std::collections::HashSet::new();
    for path in paths {
      let resolved = helpers::resolve_home(path);
      if !resolved_paths.insert(resolved.clone()) {
        return Err(DotfilesPathError::DuplicatePath(resolved));
      }
    }

    Ok(())
  }
}

impl From<PathBuf> for DotfilesPath {
  fn from(path: PathBuf) -> Self {
    DotfilesPath::Single(path)
  }
}

impl From<Vec<PathBuf>> for DotfilesPath {
  fn from(paths: Vec<PathBuf>) -> Self {
    if paths.len() == 1 {
      DotfilesPath::Single(paths.into_iter().next().unwrap())
    } else {
      DotfilesPath::Multiple(paths)
    }
  }
}

impl From<&str> for DotfilesPath {
  fn from(path: &str) -> Self {
    DotfilesPath::Single(PathBuf::from(path))
  }
}

impl AsRef<Path> for DotfilesPath {
  fn as_ref(&self) -> &Path {
    self.first_path().as_ref()
  }
}

/// Helper type for deserializing either a single path or multiple paths
#[derive(Deserialize)]
#[serde(untagged)]
enum PathOrPaths {
  Single(PathBuf),
  Multiple(Vec<PathBuf>),
}

impl From<PathOrPaths> for DotfilesPath {
  fn from(pop: PathOrPaths) -> Self {
    match pop {
      PathOrPaths::Single(path) => DotfilesPath::Single(path),
      PathOrPaths::Multiple(paths) => DotfilesPath::from(paths),
    }
  }
}

impl<'de> Deserialize<'de> for DotfilesPath {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    PathOrPaths::deserialize(deserializer)
      .map(Into::into)
      .and_then(|dotfiles_path: DotfilesPath| dotfiles_path.validate().map_err(serde::de::Error::custom).map(|_| dotfiles_path))
  }
}

/// Errors related to DotfilesPath validation
#[derive(thiserror::Error, Debug)]
pub enum DotfilesPathError {
  #[error("No dotfiles paths specified")]
  EmptyPaths,
  #[error("Duplicate dotfiles path: {0}")]
  DuplicatePath(PathBuf),
  #[error("Invalid dotfiles path: {0}")]
  InvalidPath(PathBuf),
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
  /// Path(s) to the local dotfiles
  pub(crate) dotfiles: DotfilesPath,

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

impl Config {
  /// Get all dotfiles paths
  pub fn dotfiles_paths(&self) -> &[PathBuf] {
    self.dotfiles.paths()
  }

  /// Get the primary (first) dotfiles path
  pub fn primary_dotfiles_path(&self) -> &PathBuf {
    self.dotfiles.first_path()
  }

  /// Check if using multiple dotfiles directories
  pub fn is_multi_dotfiles(&self) -> bool {
    self.dotfiles.is_multiple()
  }

  /// Get the number of dotfiles directories
  pub fn dotfiles_count(&self) -> usize {
    self.dotfiles.len()
  }

  /// Resolve home directory in all dotfiles paths
  pub fn resolve_dotfiles_homes(&mut self) {
    match &mut self.dotfiles {
      DotfilesPath::Single(path) => {
        *path = helpers::resolve_home(&*path);
      }
      DotfilesPath::Multiple(paths) => {
        for path in paths.iter_mut() {
          *path = helpers::resolve_home(&*path);
        }
      }
    }
  }
}

impl Default for Config {
  fn default() -> Self {
    Self {
      dotfiles: DotfilesPath::Single(USER_DIRS.home_dir().join(".dotfiles")),
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

#[cfg_attr(feature = "profiling", instrument)]
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

#[cfg_attr(feature = "profiling", instrument)]
fn serialize_config(config: &(impl Serialize + Debug), format: FileFormat) -> Result<String, helpers::ParseError> {
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
  #[cfg_attr(feature = "profiling", instrument)]
  pub fn new(name: &str, content: &str) -> Self {
    let pat = format!("{name}: ");
    let span: SourceSpan = if content.starts_with(&pat) {
      (0, pat.len()).into()
    } else {
      let starts = content.match_indices(&format!("\n{pat}")).collect::<Vec<_>>();
      if starts.len() == 1 { (starts[0].0 + 1, pat.len()).into() } else { (0, content.len()).into() }
    };

    Self { name: name.to_owned(), span }
  }
}

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum Error {
  #[error("Could not serialize config")]
  #[diagnostic(code(config::serialize))]
  SerializingConfig(
    #[source]
    #[diagnostic_source]
    helpers::ParseError,
  ),

  #[error("Could not write config")]
  #[diagnostic(code(config::write))]
  WritingConfig(PathBuf, #[source] std::io::Error),

  #[error("Could not get absolute path")]
  #[diagnostic(code(config::canonicalize))]
  Canonicalize(#[source] std::io::Error),

  #[error("Config file already exists")]
  #[diagnostic(code(config::exists))]
  AlreadyExists(#[label] Option<SourceSpan>, #[source_code] NamedSource<String>, #[related] Vec<AlreadyExistsError>),

  #[error("Could not parse dotfiles directory \"{0}\"")]
  #[diagnostic(code(config::filename::parse), help("Did you enter a valid file?"))]
  PathParse(PathBuf),

  #[error(transparent)]
  #[diagnostic(transparent)]
  InvalidFileFormat(
    #[from]
    #[diagnostic_source]
    crate::Error,
  ),
}

#[cfg_attr(feature = "profiling", instrument)]
pub fn create_config_file(dotfiles: Option<&Path>, config_file: &Path) -> Result<(), Error> {
  let format = config_file.try_conv::<FileFormat>()?;

  if let Ok(existing_config_str) = fs::read_to_string(config_file) {
    if let Ok(existing_config) = deserialize_config(&existing_config_str, format) {
      let mut errors: Vec<AlreadyExistsError> = vec![];

      if let Some(dotfiles) = dotfiles {
        if !existing_config.dotfiles.contains_path(dotfiles) {
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

#[cfg(test)]
mod tests;

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
mod legacy_tests {
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
