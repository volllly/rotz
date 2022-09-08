use std::path::PathBuf;

use crate::helpers::Select;

use speculoos::{assert_that, prelude::*};
use velcro::hash_set;

#[test]
fn structure() {
  let dot = parse!("yaml");

  assert_that!(dot.links)
    .is_some()
    .contains_entry(&PathBuf::from("k01"), &hash_set![PathBuf::from("v01a"), PathBuf::from("v01b")]);

  assert_that!(dot.links)
    .is_some()
    .contains_entry(&PathBuf::from("k02"), &hash_set![PathBuf::from("v02a"), PathBuf::from("v02b")]);

  assert_that!(dot.installs)
    .is_some()
    .select_and(|i| &i.cmd, |mut c| c.is_equal_to(&"i01".to_owned()))
    .select_and(|i| &i.depends, |mut d| d.contains(PathBuf::from("d01")));

  assert_that!(dot.depends).is_some().contains(&PathBuf::from("d02"));
}
