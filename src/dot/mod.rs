use std::{
  collections::{HashMap, HashSet},
  fs,
  path::{Path, PathBuf},
};

use crossterm::style::Stylize;
use itertools::Itertools;
use miette::{Diagnostic, NamedSource, SourceSpan};
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
  config::Config,
  helpers::{self, os},
  templating::{self, Parameters},
  FileFormat, FILE_EXTENSIONS_GLOB,
};

mod defaults;
mod repr;

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

#[cfg_attr(feature = "profiling", instrument)]
fn from_str_with_defaults(s: &str, format: FileFormat, defaults: Option<&CapabilitiesCanonical>) -> Result<Dot, Vec<helpers::ParseError>> {
  let repr::DotCanonical {
    global,
    windows,
    linux,
    darwin,
    windows_linux,
    linux_darwin,
    darwin_windows,
  } = repr::DotCanonical::parse(s, format)?;
  let mut capabilities: Option<CapabilitiesCanonical> = defaults.cloned().merge(global);

  if os::OS.is_windows() {
    capabilities = capabilities.merge(windows_linux);
    capabilities = capabilities.merge(darwin_windows);
    capabilities = capabilities.merge(windows);
  } else if os::OS.is_linux() {
    capabilities = capabilities.merge(windows_linux);
    capabilities = capabilities.merge(linux_darwin);
    capabilities = capabilities.merge(linux);
  } else if os::OS.is_darwin() {
    capabilities = capabilities.merge(linux_darwin);
    capabilities = capabilities.merge(darwin_windows);
    capabilities = capabilities.merge(darwin);
  }

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

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum Error {
  #[error("Could not get relative dot directory")]
  #[diagnostic(code(dotfiles::filename::strip))]
  PathStrip(#[source] std::path::StripPrefixError),

  #[error("Could not read file {0}")]
  #[diagnostic(code(dot::read))]
  ReadingDot(PathBuf, #[source] std::io::Error),

  #[error("Error walking dotfiles")]
  #[diagnostic(code(dotfiles::walk))]
  WalkingDotfiles(#[source] walkdir::Error),

  #[error("The \"dots.{0}\" file is deprecated and support will be removed in a future version.")]
  #[diagnostic(code(dot::defaults::deprecated), severity(warning), help("Please rename this file to \"defaults.{0}\"."))]
  DotsDeprecated(String, #[label] SourceSpan, #[source_code] String),

  #[cfg(feature = "yaml")]
  #[error("Could not parse dot")]
  #[diagnostic(code(dot::parse))]
  ParseDot(#[source_code] NamedSource, #[label] SourceSpan, #[related] Vec<helpers::ParseError>),

  #[cfg(feature = "yaml")]
  #[error("Could not render template for dot")]
  #[diagnostic(code(dot::render))]
  RenderDot(#[source_code] NamedSource, #[label] SourceSpan, #[source] templating::Error),

  #[error("Io Error on file \"{0}\"")]
  #[diagnostic(code(io::generic))]
  Io(PathBuf, #[source] std::io::Error),

  #[error("Could not parse dependency path \"{0}\"")]
  #[diagnostic(code(glob::parse))]
  ParseDependency(PathBuf, #[source] std::io::Error),

  #[error("Could not parse dot name \"{0}\"")]
  #[diagnostic(code(glob::parse))]
  ParseName(String, #[source] std::io::Error),

  #[error(transparent)]
  MultipleErrors(#[from] helpers::MultipleErrors),
}

#[cfg_attr(feature = "profiling", instrument(skip(engine)))]
pub(crate) fn read_dots(dotfiles_path: &Path, dots: &[String], config: &Config, engine: &templating::Engine<'_>) -> miette::Result<Vec<(String, Dot)>> {
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
            .into()
        }
      };

      let defaults = if let Some((defaults, format)) = defaults.for_path(&name) {
        match engine.render(defaults, &parameters) {
          Ok(rendered) => match repr::DotCanonical::parse(&rendered, *format) {
            Ok(parsed) => Into::<CapabilitiesCanonical>::into(parsed).into(),
            Err(err) => return Error::ParseDot(NamedSource::new(defaults, defaults.to_string()), (0, defaults.len()).into(), err).pipe(Err).into(),
          },
          Err(err) => {
            return Error::RenderDot(NamedSource::new(format!("{name}/dot.{format}"), defaults.to_string()), (0, defaults.len()).into(), err)
              .pipe(Err)
              .into()
          }
        }
      } else {
        None
      };

      match from_str_with_defaults(&text, format, defaults.as_ref()) {
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
    println!("Warning: {}", "No dots found".yellow());
    return vec![].pipe(Ok);
  }

  dots.pipe(Ok)
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
