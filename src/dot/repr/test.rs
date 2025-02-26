use speculoos::{assert_that, prelude::*};

use super::{DotComplex, InstallsComplex};

#[test]
fn parse_dot_complex() {
  let dot_string = r"
  global:
    installs: test
  ";

  let dot = DotComplex::parse(dot_string, crate::FileFormat::Yaml).unwrap();
  assert_that!(dot.filters.contains_key("global")).is_true();
  assert_that!(dot.filters.get("global").unwrap().installs)
    .is_some()
    .matches(|i| matches!(i, InstallsComplex::Simple(s) if s == "test"));
}
