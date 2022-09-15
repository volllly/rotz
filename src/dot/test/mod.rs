use std::path::Path;

use speculoos::prelude::*;
use tap::Tap;

use super::read_dots;
use crate::helpers::Select;

mod data;

#[test]
fn read_all_dots() {
  crate::templating::test::init_handlebars();

  let dots = read_dots(
    Path::new(file!()).parent().unwrap().join("data/directory_structure").as_path(),
    &["/**".to_owned()],
    &Default::default(),
  )
  .unwrap();

  assert_that!(dots)
    .tap_mut(|d| d.has_length(4))
    .tap_mut(|d| d.mapped_contains(|d| &d.0, &"/test01"))
    .tap_mut(|d| d.mapped_contains(|d| &d.0, &"/test02"))
    .tap_mut(|d| d.mapped_contains(|d| &d.0, &"/test03/test04"))
    .tap_mut(|d| d.mapped_contains(|d| &d.0, &"/test03/test05"));

  assert_that!(dots.iter().find(|d| d.0 == "/test02"))
    .is_some()
    .select(|d| &d.1.depends)
    .is_some()
    .contains(&"/test01".to_owned());

  assert_that!(dots.iter().find(|d| d.0 == "/test03/test04"))
    .is_some()
    .select(|d| &d.1.depends)
    .is_some()
    .contains(&"/test02".to_owned());

  assert_that!(dots.iter().find(|d| d.0 == "/test03/test05"))
    .is_some()
    .select(|d| &d.1.depends)
    .is_some()
    .contains(&"/test03/test04".to_owned());
}

#[test]
fn read_sub_dots() {
  crate::templating::test::init_handlebars();

  let dots = read_dots(
    Path::new(file!()).parent().unwrap().join("data/directory_structure").as_path(),
    &["/test03/*".to_owned()],
    &Default::default(),
  )
  .unwrap();

  assert_that!(dots)
    .tap_mut(|d| d.has_length(2))
    .tap_mut(|d| d.mapped_contains(|d| &d.0, &"/test03/test04"))
    .tap_mut(|d| d.mapped_contains(|d| &d.0, &"/test03/test05"));
}

#[test]
fn read_non_sub_dots() {
  crate::templating::test::init_handlebars();

  let dots = read_dots(Path::new(file!()).parent().unwrap().join("data/directory_structure").as_path(), &["/*".to_owned()], &Default::default()).unwrap();

  assert_that!(dots).has_length(2);
  assert_that!(dots).mapped_contains(|d| &d.0, &"/test01");
  assert_that!(dots).mapped_contains(|d| &d.0, &"/test02");
}

#[test]
fn read_all_file_formats() {
  crate::templating::test::init_handlebars();

  let dots = read_dots(Path::new(file!()).parent().unwrap().join("data/file_formats").as_path(), &["/**".to_owned()], &Default::default()).unwrap();

  assert_that!(dots).has_length(3);
  assert_that!(dots).mapped_contains(|d| &d.0, &"/test01");
  assert_that!(dots).mapped_contains(|d| &d.0, &"/test02");
  assert_that!(dots).mapped_contains(|d| &d.0, &"/test03");
}
