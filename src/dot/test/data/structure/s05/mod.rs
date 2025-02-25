use speculoos::{assert_that, prelude::*};

use super::{get_handlebars, get_parameters};

#[test]
fn structure() {
  let dot = parse!("yaml", &get_handlebars(), &get_parameters());

  assert_that!(dot.installs).is_none();
}
