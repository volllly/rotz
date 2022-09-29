use speculoos::{assert_that, prelude::*};

#[test]
fn structure() {
  let dot = parse!("yaml");

  assert_that!(dot.installs).is_none();
}
