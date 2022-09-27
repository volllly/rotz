use std::path::PathBuf;

use speculoos::{assert_that, prelude::*};
use tap::Tap;
use velcro::hash_set;

#[test]
fn structure() {
  let dot = parse!("yaml");

  assert_that!(dot.links)
    .is_some()
    .tap_mut(|l| l.contains_entry(&PathBuf::from("k01"), &hash_set![PathBuf::from("v01")]))
    .tap_mut(|l| l.contains_entry(&PathBuf::from("k02"), &hash_set![PathBuf::from("v02a"), PathBuf::from("v02b")]));
}
