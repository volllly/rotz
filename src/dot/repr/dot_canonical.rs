use std::str::FromStr;

#[cfg(test)]
use fake::Dummy;
use indexmap::IndexMap;
#[cfg(feature = "profiling")]
use tracing::instrument;

#[cfg(test)]
use super::IndexMapFaker;
use crate::{FileFormat, helpers};

use super::{CapabilitiesCanonical, DotComplex, Filters};

#[derive(Debug, Default, Clone)]
#[cfg_attr(test, derive(Dummy))]
pub struct DotCanonical {
  #[cfg_attr(test, dummy(faker = "IndexMapFaker"))]
  pub filters: IndexMap<Filters, CapabilitiesCanonical>,
}

impl TryFrom<DotComplex> for DotCanonical {
  type Error = Vec<chumsky::error::Simple<char>>;
  #[cfg_attr(feature = "profiling", instrument)]
  fn try_from(value: DotComplex) -> Result<Self, Self::Error> {
    let mut errors = Self::Error::new();
    let mut filters = IndexMap::new();
    for (filter, dot) in value.filters {
      match Filters::from_str(&filter) {
        Ok(f) => {
          filters.insert(f, dot.into());
        }
        Err(e) => {
          errors.extend_from_slice(&e);
        }
      }
    }
    if !errors.is_empty() {
      return Err(errors);
    }
    Ok(Self { filters })
  }
}

impl DotCanonical {
  #[cfg_attr(feature = "profiling", instrument)]
  pub(crate) fn parse(value: &str, format: FileFormat) -> Result<Self, Vec<helpers::ParseError>> {
    DotComplex::parse(value, format).map(TryInto::try_into).map(|d| d.unwrap())
  }
}
