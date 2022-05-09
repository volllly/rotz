use std::{error::Error, fmt::Debug};

use miette::{miette, Diagnostic, Result};
use somok::Somok;

#[derive(thiserror::Error, Diagnostic, Debug)]
#[error("Encountered multiple errors")]
pub struct MultipleErrors(#[related] Vec<miette::Error>);

pub fn join_err_result<T, E>(result: Vec<Result<T, E>>) -> Result<Vec<T>, MultipleErrors>
where
  T: Debug,
  E: Error + Send + Sync + 'static,
{
  if result.iter().any(|p| p.is_err()) {
    MultipleErrors(result.into_iter().filter(Result::is_err).map(Result::unwrap_err).map(|e| miette!(e)).collect::<Vec<_>>()).error()
  } else {
    Ok(result.into_iter().map(Result::unwrap).collect())
  }
}

#[cfg_attr(all(nightly, coverage), no_coverage)]
pub fn _join_err<E>(result: Vec<E>) -> Result<(), MultipleErrors>
where
  E: Error + Send + Sync + 'static,
{
  if result.is_empty() {
    return ().okay();
  };

  MultipleErrors(result.into_iter().map(|e| miette!(e)).collect::<Vec<_>>()).error()
}

#[cfg(test)]
mod tests {
  use speculoos::prelude::*;

  use crate::helpers::join_err_result;

  #[derive(thiserror::Error, Debug)]
  #[error("")]
  struct Error();

  #[test]
  fn join_err_result_none() {
    let joined = join_err_result(vec![Ok::<(), Error>(()), Ok::<(), Error>(())]);
    assert_that!(&joined).is_ok().has_length(2);
  }

  #[test]
  fn join_err_result_some() {
    let joined = join_err_result(vec![Ok::<(), Error>(()), Err::<(), Error>(Error()), Err::<(), Error>(Error()), Ok::<(), Error>(())]);

    assert_that!(&joined).is_err().map(|e| &e.0).has_length(2);
  }
}
