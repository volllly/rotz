use speculoos::{assert_that, prelude::*};

use super::DotComplex;

#[test]
fn serde_test() {
  use indexmap::IndexMap;
  use serde::Deserialize;

  #[derive(Debug, Deserialize)]
  struct Values {
    key: String,
  }

  #[derive(Debug, Deserialize)]
  struct Test {
    #[serde(flatten)]
    properties: IndexMap<String, Values>,
  }

  dbg!(serde_yaml::from_str::<Test>(
    r"
    test1:
      key: key1
    
    test2:
      key: key1
    
    test3:
      key: key1 
    "
  ))
  .ok();
}
#[test]
fn parse_dot_complex() {
  let dot_string = r"
  global:
    installs: test
  ";

  let dot = DotComplex::parse(dot_string, crate::FileFormat::Yaml).unwrap();
  assert_that!(dot.filters.contains_key("global")).is_true();
}
