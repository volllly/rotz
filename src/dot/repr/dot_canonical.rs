use std::str::FromStr;

#[cfg(test)]
use fake::Dummy;
use indexmap::IndexMap;
use tap::TryConv;
#[cfg(feature = "profiling")]
use tracing::instrument;

#[cfg(test)]
use super::IndexMapFaker;
use crate::{FileFormat, helpers};

use super::{CapabilitiesCanonical, DotComplex, Selectors};

#[derive(Debug, Default, Clone)]
#[cfg_attr(test, derive(Dummy))]
pub struct DotCanonical {
  #[cfg_attr(test, dummy(faker = "IndexMapFaker"))]
  pub selectors: IndexMap<Selectors, CapabilitiesCanonical>,
}

impl TryFrom<DotComplex> for DotCanonical {
  type Error = Vec<helpers::ParseError>;
  #[cfg_attr(feature = "profiling", instrument)]
  fn try_from(value: DotComplex) -> Result<Self, Self::Error> {
    let mut errors = Self::Error::new();
    let mut selectors = IndexMap::new();
    for (selector, dot) in value.selectors {
      match Selectors::from_str(&selector) {
        Ok(f) => {
          selectors.insert(f, dot.into());
        }
        Err(e) => {
          errors.push(helpers::ParseError::Selector(e));
        }
      }
    }
    if !errors.is_empty() {
      return Err(errors);
    }
    Ok(Self { selectors })
  }
}

impl DotCanonical {
  #[cfg_attr(feature = "profiling", instrument)]
  pub(crate) fn parse(value: &str, format: FileFormat) -> Result<Self, Vec<helpers::ParseError>> {
    DotComplex::parse(value, format)?.try_conv::<DotCanonical>()
  }
}
