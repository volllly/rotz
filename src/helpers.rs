use std::{
  ffi::OsStr,
  fmt::Debug,
  io::{self, Write},
  path::{Path, PathBuf},
  process,
};

use crate::{FileFormat, FILE_EXTENSIONS};
use itertools::Itertools;
use miette::{Diagnostic, Result};
use somok::Somok;
#[cfg(test)]
use speculoos::assert_that;
use wax::{Any, BuildError, Glob};

#[derive(thiserror::Error, Diagnostic, Debug)]
#[error("Encountered multiple errors")]
pub struct MultipleErrors(#[related] Vec<miette::Error>);

pub fn join_err_result<T, E>(result: Vec<Result<T, E>>) -> Result<Vec<T>, MultipleErrors>
where
  T: Debug,
  E: miette::Diagnostic + Send + Sync + 'static,
{
  if result.iter().any(std::result::Result::is_err) {
    MultipleErrors(result.into_iter().filter(Result::is_err).map(Result::unwrap_err).map(miette::Error::new).collect_vec()).error()
  } else {
    Ok(result.into_iter().map(Result::unwrap).collect())
  }
}

#[cfg_attr(all(nightly, coverage), no_coverage)]
pub fn _join_err(result: Vec<miette::Error>) -> Result<(), MultipleErrors> {
  if result.is_empty() {
    return ().okay();
  };

  MultipleErrors(result.into_iter().collect_vec()).error()
}

pub mod os {
  use derive_more::{Display, IsVariant};

  #[derive(IsVariant, Display)]
  #[allow(dead_code)]
  pub enum Os {
    Windows,
    Linux,
    Darwin,
  }

  #[cfg(windows)]
  pub const OS: Os = Os::Windows;
  #[cfg(all(not(target_os = "macos"), unix))]
  pub const OS: Os = Os::Linux;
  #[cfg(target_os = "macos")]
  pub const OS: Os = Os::Darwin;
}

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum RunError {
  #[error("Could not spawn command")]
  #[diagnostic(code(process::command::spawn))]
  Spawn(#[source] io::Error),

  #[error("Command did not complete successfully. (Exitcode {0:?})")]
  #[diagnostic(code(process::command::execute))]
  Execute(Option<i32>),

  #[error("Could not write output")]
  #[diagnostic(code(process::command::output))]
  Write(#[from] io::Error),
}

pub fn run_command(cmd: &str, args: &[impl AsRef<OsStr>], silent: bool, dry_run: bool) -> Result<(), RunError> {
  if dry_run {
    return ().okay();
  }

  let output = process::Command::new(cmd).args(args).stdin(process::Stdio::null()).output().map_err(RunError::Spawn)?;

  if !silent {
    std::io::stdout().write_all(&output.stdout)?;
    std::io::stdout().write_all(&output.stderr)?;
  }

  if !output.status.success() {
    if silent {
      std::io::stdout().write_all(&output.stdout)?;
      std::io::stdout().write_all(&output.stderr)?;
    }
    RunError::Execute(output.status.code()).error()?;
  };

  ().okay()
}

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum GlobError {
  #[error("Could not build GlobSet")]
  #[diagnostic(code(glob::set::parse))]
  Build(#[from] wax::BuildError<'static>),
}

pub fn glob_from_vec(from: &[String], postfix: &str) -> miette::Result<Any<'static>> {
  let globs = from
    .iter()
    .map(|g| format!("{}{}", g, postfix))
    .map(|g| Glob::new(&g).map(Glob::into_owned).map_err(|e| GlobError::Build(BuildError::into_owned(e))))
    .collect_vec();

  wax::any::<'static, Glob, _>(join_err_result(globs)?).unwrap().okay()
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) fn get_file_with_format(path: impl AsRef<Path>, base_name: impl AsRef<Path>) -> Option<(PathBuf, FileFormat)> {
  FILE_EXTENSIONS.iter().map(|e| (path.as_ref().join(base_name.as_ref().with_extension(e.0)), e.1)).find(|e| e.0.exists())
}

#[cfg(test)]
pub trait Select<'s, O: 's, N: 's> {
  fn select<F>(self, selector: F) -> speculoos::Spec<'s, N>
  where
    F: Fn(&'s O) -> &'s N;

  fn select_and<S, W>(&self, selector: S, with: W) -> &speculoos::Spec<'s, O>
  where
    S: Fn(&'s O) -> &'s N,
    W: Fn(speculoos::Spec<'s, N>);

  fn and<F>(self, and: F) -> speculoos::Spec<'s, O>
  where
    F: Fn(&speculoos::Spec<'s, O>);
}

#[cfg(test)]
impl<'s, O: 's, N: 's> Select<'s, O, N> for speculoos::Spec<'s, O> {
  fn select<F>(self, selector: F) -> speculoos::Spec<'s, N>
  where
    F: Fn(&'s O) -> &'s N,
  {
    assert_that!(*selector(self.subject))
  }

  fn select_and<S, W>(&self, selector: S, with: W) -> &speculoos::Spec<'s, O>
  where
    S: Fn(&'s O) -> &'s N,
    W: Fn(speculoos::Spec<'s, N>),
  {
    with(assert_that!(*selector(self.subject)));
    self
  }

  fn and<F>(self, and: F) -> speculoos::Spec<'s, O>
  where
    F: Fn(&speculoos::Spec<'s, O>),
  {
    and(&self);
    self
  }
}

#[cfg(test)]
mod tests {
  use miette::Diagnostic;
  use speculoos::prelude::*;

  use crate::helpers::join_err_result;

  #[derive(thiserror::Error, Debug, Diagnostic)]
  #[error("")]
  struct Error;

  #[test]
  fn join_err_result_none() {
    let joined = join_err_result(vec![Ok::<(), Error>(()), Ok::<(), Error>(())]);
    assert_that!(&joined).is_ok().has_length(2);
  }

  #[test]
  fn join_err_result_some() {
    let joined = join_err_result(vec![Ok::<(), Error>(()), Err::<(), Error>(Error), Err::<(), Error>(Error), Ok::<(), Error>(())]);

    assert_that!(&joined).is_err().map(|e| &e.0).has_length(2);
  }
}
