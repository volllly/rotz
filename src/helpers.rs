use std::{error::Error, fmt::Debug};

use color_eyre::eyre::{eyre, Result};
use somok::Somok;

#[allow(clippy::type_complexity)]
pub fn join_err_result<T, E: Error + Send + Sync + 'static>(result: Vec<Result<T, E>>) -> Result<Vec<T>>
where
  T: Debug,
{
  if result.iter().any(|p| p.is_err()) {
    result
      .into_iter()
      .filter(Result::is_err)
      .map(Result::unwrap_err)
      .fold(Err(eyre!("encountered multiple errors")), |report, e| color_eyre::Help::error(report, e))
  } else {
    Ok(result.into_iter().map(Result::unwrap).collect())
  }
}

#[allow(clippy::type_complexity)]
pub fn join_err<E: Error + Send + Sync + 'static>(result: Vec<E>) -> Result<()> {
  if result.is_empty() {
    return ().okay();
  };

  result.into_iter().fold(Err(eyre!("encountered multiple errors")), |report, e| color_eyre::Help::error(report, e))
}
