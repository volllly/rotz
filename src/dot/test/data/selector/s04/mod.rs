use speculoos::{assert_that, prelude::*};

use crate::helpers::Select;

use super::{get_handlebars, get_parameters};

#[test]
fn selectors() {
  let dot = crate::parse!("yaml", &get_handlebars(), &get_parameters());

  assert_that!(dot.installs).is_some().select(|i| &i.cmd).is_equal_to("i02".to_owned());
}
