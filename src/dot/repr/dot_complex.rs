#[cfg(test)]
use super::IndexMapFaker;
use super::{CapabilitiesComplex, DotSimplified, parse_inner};
use crate::{FileFormat, helpers};
#[cfg(test)]
use fake::Dummy;
use indexmap::IndexMap;
use serde::Deserialize;
use tap::{Conv, Pipe};
#[cfg(feature = "profiling")]
use tracing::instrument;

#[derive(Deserialize, Debug, Default, Clone)]
#[cfg_attr(test, derive(Dummy))]
pub(super) struct DotComplex {
  #[cfg_attr(test, dummy(faker = "IndexMapFaker"))]
  #[serde(flatten)]
  pub filters: IndexMap<String, CapabilitiesComplex>,
}

impl DotComplex {
  #[cfg_attr(feature = "profiling", instrument)]
  pub(super) fn parse(value: &str, format: FileFormat) -> Result<Self, Vec<helpers::ParseError>> {
    match parse_inner::<Self>(value, format) {
      Ok(parsed) => parsed.pipe(Ok),
      Err(err) => Self {
        filters: IndexMap::from([(
          "global".to_owned(),
          parse_inner::<DotSimplified>(value, format).map_err(|e| vec![err, e])?.conv::<CapabilitiesComplex>(),
        )]),
      }
      .pipe(Ok),
    }
  }
}
