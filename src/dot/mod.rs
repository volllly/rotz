use std::{
  collections::{HashMap, HashSet},
  fs,
  path::{Path, PathBuf},
};

use crossterm::style::Stylize;
use itertools::Itertools;
use miette::NamedSource;
use path_slash::PathBufExt;
use tap::{Pipe, TryConv};
#[cfg(feature = "profiling")]
use tracing::instrument;
use walkdir::WalkDir;
use wax::Pattern;

use self::{
  defaults::Defaults,
  repr::{CapabilitiesCanonical, Merge},
};
use crate::{
  FILE_EXTENSIONS_GLOB, FileFormat,
  config::Config,
  helpers,
  templating::{self, Engine, Parameters},
};

mod defaults;
mod error;
mod repr;
pub use error::Error;

#[derive(Clone, Debug)]
pub struct Installs {
  pub(crate) cmd: String,
  pub(crate) depends: HashSet<String>,
}

impl From<repr::InstallsCanonical> for Option<Installs> {
  fn from(from: repr::InstallsCanonical) -> Self {
    match from {
      repr::InstallsCanonical::None(_) => None,
      repr::InstallsCanonical::Full { cmd, depends } => Installs { cmd, depends }.pipe(Some),
    }
  }
}

#[derive(Default, Clone, Debug)]
pub struct Dot {
  pub(crate) links: Option<HashMap<PathBuf, HashSet<PathBuf>>>,
  pub(crate) installs: Option<Installs>,
  pub(crate) depends: Option<HashSet<String>>,
}

#[cfg_attr(feature = "profiling", instrument(skip(engine)))]
fn from_str_with_defaults(s: &str, format: FileFormat, defaults: Option<&CapabilitiesCanonical>, engine: &Engine<'_>, parameters: &Parameters<'_>) -> Result<Dot, Vec<helpers::ParseError>> {
  let capabilities: Option<CapabilitiesCanonical> = defaults
    .cloned()
    .merge(CapabilitiesCanonical::from(repr::DotCanonical::parse(s, format)?, engine, parameters).map_err(|e| vec![e])?.pipe(Some));
  if let Some(capabilities) = capabilities {
    Dot {
      links: capabilities.links,
      installs: capabilities.installs.and_then(Into::into),
      depends: capabilities.depends,
    }
  } else {
    Dot::default()
  }
  .pipe(Ok)
}

#[cfg_attr(feature = "profiling", instrument(skip(engine)))]
pub(crate) fn read_dots(dotfiles_path: &Path, dots: &[String], config: &Config, engine: &templating::Engine<'_>) -> miette::Result<Vec<(String, Dot)>> {
  // Backward compatibility: convert single path to multiple paths and call multi version
  read_dots_multi(&[dotfiles_path.to_path_buf()], dots, config, engine)
}

/// Read dots from multiple dotfiles directories with conflict resolution
#[cfg_attr(feature = "profiling", instrument(skip(engine)))]
pub(crate) fn read_dots_multi(dotfiles_paths: &[std::path::PathBuf], dots: &[String], config: &Config, engine: &templating::Engine<'_>) -> miette::Result<Vec<(String, Dot)>> {
  use std::collections::HashMap;

  if dotfiles_paths.is_empty() {
    println!("Warning: {}", "No dotfiles paths provided".dark_yellow());
    return vec![].pipe(Ok);
  }

  let mut all_dots: HashMap<String, (Dot, std::path::PathBuf)> = HashMap::new();
  let mut conflicts: Vec<(String, Vec<std::path::PathBuf>)> = Vec::new();

  for (_priority, dotfiles_path) in dotfiles_paths.iter().enumerate() {
    if !dotfiles_path.exists() {
      println!("Warning: Dotfiles directory does not exist: {}", dotfiles_path.display().to_string().dark_yellow());
      continue;
    }

    match read_dots_from_single_path(dotfiles_path, dots, config, engine) {
      Ok(dots_from_path) => {
        for (dot_name, dot) in dots_from_path {
          if let Some((_, existing_source)) = all_dots.get(&dot_name) {
            // Conflict detected - track it
            let conflict_sources = vec![existing_source.clone(), dotfiles_path.clone()];
            if let Some((_, existing_conflicts)) = conflicts.iter_mut().find(|(name, _)| name == &dot_name) {
              if !existing_conflicts.contains(dotfiles_path) {
                existing_conflicts.push(dotfiles_path.clone());
              }
            } else {
              conflicts.push((dot_name.clone(), conflict_sources));
            }

            // First directory wins (priority 0 beats priority 1, etc.)
            // Don't overwrite existing dot
            continue;
          } else {
            // No conflict, add the dot
            all_dots.insert(dot_name, (dot, dotfiles_path.clone()));
          }
        }
      }
      Err(e) => {
        println!(
          "Warning: Failed to read dots from {}: {}",
          dotfiles_path.display().to_string().dark_yellow(),
          e.to_string().dark_yellow()
        );
        // Continue processing other directories
      }
    }
  }

  // Log conflicts
  for (dot_name, conflicting_paths) in conflicts {
    println!(
      "Info: Dot '{}' found in multiple directories (using first): {}",
      dot_name.dark_blue(),
      conflicting_paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ").dark_green()
    );
  }

  if all_dots.is_empty() {
    println!("Warning: {}", "No dots found in any directory".dark_yellow());
    return vec![].pipe(Ok);
  }

  // Convert back to the expected format
  let result: Vec<(String, Dot)> = all_dots.into_iter().map(|(name, (dot, _))| (name, dot)).collect();

  println!(
    "Info: Loaded {} dots from {} directories",
    result.len().to_string().dark_green(),
    dotfiles_paths.len().to_string().dark_green()
  );

  result.pipe(Ok)
}

/// Read dots from a single dotfiles directory (helper function)
#[cfg_attr(feature = "profiling", instrument(skip(engine)))]
fn read_dots_from_single_path(dotfiles_path: &Path, dots: &[String], config: &Config, engine: &templating::Engine<'_>) -> miette::Result<Vec<(String, Dot)>> {
  let defaults = Defaults::from_path(dotfiles_path).map_err(|e| *e)?;

  let dots = helpers::glob_from_vec(dots, format!("/dot.{FILE_EXTENSIONS_GLOB}").as_str().pipe(Some))?;

  let paths = WalkDir::new(dotfiles_path)
    .into_iter()
    .filter(|e| e.as_ref().map_or(true, |e| !e.file_type().is_dir()))
    .map(|d| -> Result<(std::string::String, std::path::PathBuf), Error> {
      let d = d.map_err(Error::WalkingDotfiles)?;
      let path = d.path().strip_prefix(dotfiles_path).map(Path::to_path_buf).map_err(Error::PathStrip)?;
      let absolutized = helpers::absolutize_virtually(&path).map_err(|e| Error::ParseName(path.to_string_lossy().to_string(), e))?;
      Ok((absolutized, path))
    })
    .filter(|e| e.as_ref().map_or(true, |e| dots.is_match(e.0.as_str())))
    .map(|e| match e {
      Ok(e) => {
        let format = e.1.as_path().try_conv::<FileFormat>().unwrap();
        (e.1, format).pipe(Ok)
      }
      Err(err) => err.pipe(Err),
    });

  let dotfiles = crate::helpers::join_err_result(paths.collect())?.into_iter().map(|p| {
    let name = p.0.parent().unwrap().to_path_buf().to_slash_lossy().to_string();
    (name, fs::read_to_string(dotfiles_path.join(&p.0)).map(|d| (d, p.1)).map_err(|e| Error::Io(dotfiles_path.join(p.0), e)))
  });

  let dots = dotfiles.filter_map(|f| match f {
    (name, Ok((text, format))) => {
      let parameters = Parameters { config, name: &name };
      let text = match engine.render(&text, &parameters) {
        Ok(text) => text,
        Err(err) => {
          return Error::RenderDot(NamedSource::new(format!("{name}/dot.{format}"), text.clone()), (0, text.len()).into(), err)
            .pipe(Err)
            .into();
        }
      };

      let defaults = if let Some((defaults, format)) = defaults.for_path(&name) {
        match engine.render(defaults, &parameters) {
          Ok(rendered) => match repr::DotCanonical::parse(&rendered, *format) {
            Ok(parsed) => match CapabilitiesCanonical::from(parsed, engine, &parameters) {
              Ok(ok) => ok,
              Err(err) => {
                return Error::ParseDot(NamedSource::new(defaults, defaults.to_string()), (0, defaults.len()).into(), vec![err])
                  .pipe(Err)
                  .into();
              }
            }
            .into(),
            Err(err) => {
              return Error::ParseDot(NamedSource::new(defaults, defaults.to_string()), (0, defaults.len()).into(), err).pipe(Err).into();
            }
          },
          Err(err) => {
            return Error::RenderDot(NamedSource::new(format!("{name}/dot.{format}"), defaults.to_string()), (0, defaults.len()).into(), err)
              .pipe(Err)
              .into();
          }
        }
      } else {
        None
      };

      match from_str_with_defaults(&text, format, defaults.as_ref(), engine, &parameters) {
        Ok(dot) => (name.clone(), dot).pipe(Ok).into(),
        Err(err) => Error::ParseDot(NamedSource::new(format!("{name}/dot.{format}"), text.clone()), (0, text.len()).into(), err)
          .pipe(Err)
          .into(),
      }
    }
    (_, Err(Error::Io(file, err))) => match err.kind() {
      std::io::ErrorKind::NotFound => None,
      _ => Error::Io(file, err).pipe(Err).into(),
    },
    (_, Err(err)) => err.pipe(Err).into(),
  });
  let dots = canonicalize_dots(crate::helpers::join_err_result(dots.collect())?)?;

  if dots.is_empty() {
    println!("Warning: {}", "No dots found".dark_yellow());
    return vec![].pipe(Ok);
  }

  dots.pipe(Ok)
}

/// Read dots using Config's dotfiles paths
#[cfg_attr(feature = "profiling", instrument(skip(engine)))]
pub(crate) fn read_dots_from_config(dots: &[String], config: &Config, engine: &templating::Engine<'_>) -> miette::Result<Vec<(String, Dot)>> {
  read_dots_multi(config.dotfiles_paths(), dots, config, engine)
}

/// Read dots with source directory information for linking
#[cfg_attr(feature = "profiling", instrument(skip(engine)))]
pub(crate) fn read_dots_with_sources(dots: &[String], config: &Config, engine: &templating::Engine<'_>) -> miette::Result<Vec<(String, Dot, std::path::PathBuf)>> {
  use std::collections::HashMap;

  if config.dotfiles_paths().is_empty() {
    println!("Warning: {}", "No dotfiles paths provided".dark_yellow());
    return vec![].pipe(Ok);
  }

  let mut all_dots: HashMap<String, (Dot, std::path::PathBuf)> = HashMap::new();
  let mut conflicts: Vec<(String, Vec<std::path::PathBuf>)> = Vec::new();

  for dotfiles_path in config.dotfiles_paths() {
    if !dotfiles_path.exists() {
      println!("Warning: Dotfiles directory does not exist: {}", dotfiles_path.display().to_string().dark_yellow());
      continue;
    }

    match read_dots_from_single_path(dotfiles_path, dots, config, engine) {
      Ok(dots_from_path) => {
        for (dot_name, dot) in dots_from_path {
          if let Some((_, existing_source)) = all_dots.get(&dot_name) {
            // Conflict detected - track it
            let conflict_sources = vec![existing_source.clone(), dotfiles_path.clone()];
            if let Some((_, existing_conflicts)) = conflicts.iter_mut().find(|(name, _)| name == &dot_name) {
              if !existing_conflicts.contains(dotfiles_path) {
                existing_conflicts.push(dotfiles_path.clone());
              }
            } else {
              conflicts.push((dot_name.clone(), conflict_sources));
            }

            // First directory wins - don't overwrite existing dot
            continue;
          } else {
            // No conflict, add the dot with its source directory
            all_dots.insert(dot_name, (dot, dotfiles_path.clone()));
          }
        }
      }
      Err(e) => {
        println!(
          "Warning: Failed to read dots from {}: {}",
          dotfiles_path.display().to_string().dark_yellow(),
          e.to_string().dark_yellow()
        );
        // Continue processing other directories
      }
    }
  }

  // Log conflicts
  for (dot_name, conflicting_paths) in conflicts {
    println!(
      "Info: Dot '{}' found in multiple directories (using first): {}",
      dot_name.dark_blue(),
      conflicting_paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ").dark_green()
    );
  }

  if all_dots.is_empty() {
    println!("Warning: {}", "No dots found in any directory".dark_yellow());
    return vec![].pipe(Ok);
  }

  // Convert to the format with source directory information
  let result: Vec<(String, Dot, std::path::PathBuf)> = all_dots.into_iter().map(|(name, (dot, source))| (name, dot, source)).collect();

  println!(
    "Info: Loaded {} dots from {} directories",
    result.len().to_string().dark_green(),
    config.dotfiles_paths().len().to_string().dark_green()
  );

  result.pipe(Ok)
}

#[cfg_attr(feature = "profiling", instrument)]
fn canonicalize_dots(dots: Vec<(String, Dot)>) -> Result<Vec<(String, Dot)>, helpers::MultipleErrors> {
  let dots = dots.into_iter().map(|mut dot| {
    let name = helpers::absolutize_virtually(Path::new(&dot.0)).map_err(|e| Error::ParseName(dot.0.clone(), e))?;

    if let Some(installs) = &mut dot.1.installs {
      let depends = installs.depends.iter().map(|dependency| {
        let dependency_base = Path::new(&name).parent().unwrap_or_else(|| Path::new("")).join(dependency);

        let dependency_base = helpers::absolutize_virtually(&dependency_base).map_err(|e| Error::ParseDependency(dependency_base, e))?;
        dependency_base.pipe(Ok::<_, Error>)
      });
      installs.depends = helpers::join_err_result(depends.collect_vec())?.into_iter().collect::<HashSet<_>>();
    }

    if let Some(depends) = &dot.1.depends {
      let depends_mapped = depends.iter().map(|dependency| {
        let dependency_base = Path::new(&name).parent().unwrap_or_else(|| Path::new("")).join(dependency);

        let dependency_base = helpers::absolutize_virtually(&dependency_base).map_err(|e| Error::ParseDependency(dependency_base, e))?;
        dependency_base.pipe(Ok::<_, Error>)
      });
      dot.1.depends = Some(helpers::join_err_result(depends_mapped.collect_vec())?.into_iter().collect::<HashSet<_>>());
    }

    (name, dot.1).pipe(Ok::<_, Error>)
  });

  helpers::join_err_result(dots.collect_vec())
}

#[cfg(test)]
mod test;

#[cfg(test)]
mod tests;
