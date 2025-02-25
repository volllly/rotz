use std::path::PathBuf;

use miette::{Diagnostic, NamedSource, SourceSpan};

use crate::{helpers, templating};

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
  ParseDot(#[source_code] NamedSource<String>, #[label] SourceSpan, #[related] Vec<helpers::ParseError>),

  #[cfg(feature = "yaml")]
  #[error("Could not render template for dot")]
  #[diagnostic(code(dot::render))]
  RenderDot(#[source_code] NamedSource<String>, #[label] SourceSpan, #[source] templating::Error),

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
