use std::path::PathBuf;

use crate::helpers::Select;

use speculoos::{assert_that, prelude::*};
use velcro::hash_set;

#[test]
fn structure() {
  let dot = parse!("yaml");

  assert_that!(dot.links).is_some().contains_entry(&PathBuf::from("k01"), &hash_set![PathBuf::from("v01")]);

  assert_that!(dot.links).is_some().contains_entry(&PathBuf::from("k02"), &hash_set![PathBuf::from("v02")]);

  assert_that!(dot.installs).is_some().select(|i| &i.cmd).is_equal_to(&"i01".to_owned());

  assert_that!(dot.depends).is_some().contains(&PathBuf::from("d01"));
}
