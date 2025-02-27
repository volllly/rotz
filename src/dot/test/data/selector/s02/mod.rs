use speculoos::{assert_that, prelude::*};

use super::{get_handlebars, get_parameters};

#[test]
fn selectors() {
  let dot = crate::parse!("yaml", &get_handlebars(), &get_parameters());

  assert_that!(dot.installs).is_none();
}
