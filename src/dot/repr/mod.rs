#[cfg(test)]
use fake::{Dummy, Fake, Faker};
use serde::Deserialize;
use tap::Pipe;
#[cfg(feature = "profiling")]
use tracing::instrument;

use crate::{FileFormat, helpers};

mod dot_simplified;
#[cfg(test)]
mod test;
use dot_simplified::DotSimplified;
mod dot_complex;
use dot_complex::DotComplex;
mod dot_canonical;
pub use dot_canonical::*;
mod capabilities_complex;
use capabilities_complex::CapabilitiesComplex;
mod capabilities_canonical;
pub use capabilities_canonical::*;
mod links_complex;
use links_complex::LinksComplex;
mod installs_complex;
use installs_complex::InstallsComplex;
mod installs_canonical;
pub use installs_canonical::*;
mod filter;
use filter::Filters;

#[cfg(feature = "toml")]
fn parse_inner_toml<T: for<'de> Deserialize<'de>>(value: &str) -> Result<T, helpers::ParseError> {
  serde_toml::from_str::<T>(value)?.pipe(Ok)
}

#[cfg(feature = "yaml")]
fn parse_inner_yaml<T: for<'de> Deserialize<'de> + Default>(value: &str) -> Result<T, helpers::ParseError> {
  match serde_yaml::from_str::<T>(value) {
    Ok(ok) => ok.pipe(Ok),
    Err(err) => match err.location() {
      Some(_) => err.pipe(Err)?,
      None => T::default().pipe(Ok),
    },
  }
}

#[cfg(feature = "json")]
fn parse_inner_json<T: for<'de> Deserialize<'de>>(value: &str) -> Result<T, helpers::ParseError> {
  serde_json::from_str::<T>(value)?.pipe(Ok)
}

#[cfg_attr(feature = "profiling", instrument)]
fn parse_inner<T: for<'de> Deserialize<'de> + Default>(value: &str, format: FileFormat) -> Result<T, helpers::ParseError> {
  match format {
    #[cfg(feature = "yaml")]
    FileFormat::Yaml => parse_inner_yaml::<T>(value),
    #[cfg(feature = "toml")]
    FileFormat::Toml => parse_inner_toml::<T>(value),
    #[cfg(feature = "json")]
    FileFormat::Json => parse_inner_json::<T>(value),
  }
}

pub trait Merge<T> {
  fn merge(self, merge: T) -> Self;
}

#[cfg(test)]
struct IndexMapFaker;

#[cfg(test)]
#[allow(clippy::implicit_hasher)]
impl<K, V> Dummy<IndexMapFaker> for indexmap::IndexMap<K, V>
where
  K: std::hash::Hash + std::cmp::Eq + Dummy<Faker>,
  V: Dummy<Faker>,
{
  fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &IndexMapFaker, rng: &mut R) -> Self {
    let mut map = Self::new();

    for _ in 0..((0..10).fake_with_rng(rng)) {
      map.insert(Faker.fake::<K>(), Faker.fake::<V>());
    }

    map
  }
}
