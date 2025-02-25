use std::path::Path;

use rstest::rstest;
use speculoos::prelude::*;
use tap::Tap;

use super::{defaults::Defaults, read_dots};
use crate::{helpers::Select, templating::test::get_handlebars};

mod data;

#[test]
fn read_all_dots() {
  let dots = read_dots(
    Path::new(file!()).parent().unwrap().join("data/directory_structure").as_path(),
    &["/**".to_owned()],
    &Default::default(),
    &get_handlebars(),
  )
  .unwrap();

  assert_that!(dots)
    .tap_mut(|d| d.has_length(5))
    .tap_mut(|d| d.mapped_contains(|d| &d.0, &"/test01"))
    .tap_mut(|d| d.mapped_contains(|d| &d.0, &"/test02"))
    .tap_mut(|d| d.mapped_contains(|d| &d.0, &"/test03/test04"))
    .tap_mut(|d| d.mapped_contains(|d| &d.0, &"/test03/test05"))
    .tap_mut(|d| d.mapped_contains(|d| &d.0, &"/test03/test06"));

  assert_that!(dots.iter().find(|d| d.0 == "/test02"))
    .is_some()
    .select(|d| &d.1.depends)
    .is_some()
    .contains("/test01".to_owned());

  assert_that!(dots.iter().find(|d| d.0 == "/test03/test04"))
    .is_some()
    .select(|d| &d.1.depends)
    .is_some()
    .contains("/test02".to_owned());

  assert_that!(dots.iter().find(|d| d.0 == "/test03/test05"))
    .is_some()
    .select(|d| &d.1.depends)
    .is_some()
    .contains("/test03/test04".to_owned());
}

#[test]
fn read_sub_dots() {
  let dots = read_dots(
    Path::new(file!()).parent().unwrap().join("data/directory_structure").as_path(),
    &["/test03/*".to_owned()],
    &Default::default(),
    &get_handlebars(),
  )
  .unwrap();

  assert_that!(dots)
    .tap_mut(|d| d.has_length(3))
    .tap_mut(|d| d.mapped_contains(|d| &d.0, &"/test03/test04"))
    .tap_mut(|d| d.mapped_contains(|d| &d.0, &"/test03/test05"))
    .tap_mut(|d| d.mapped_contains(|d| &d.0, &"/test03/test06"));
}

#[test]
fn read_non_sub_dots() {
  let dots = read_dots(
    Path::new(file!()).parent().unwrap().join("data/directory_structure").as_path(),
    &["/*".to_owned()],
    &Default::default(),
    &get_handlebars(),
  )
  .unwrap();

  assert_that!(dots).has_length(2);
  assert_that!(dots).mapped_contains(|d| &d.0, &"/test01");
  assert_that!(dots).mapped_contains(|d| &d.0, &"/test02");
}

#[test]
fn read_defaults() {
  let defaults = Defaults::from_path(Path::new(file!()).parent().unwrap().join("data/directory_structure").as_path()).unwrap();

  assert_that!(defaults.for_path("/test03/test05")).is_some();
  assert_that!(defaults.for_path("/test03/test04")).is_some();
  assert_that!(defaults.for_path("/test03")).is_some();
}

#[test]
fn read_sub_dots_with_defaults() {
  let dots = read_dots(
    Path::new(file!()).parent().unwrap().join("data/directory_structure").as_path(),
    &["/test03/*".to_owned()],
    &Default::default(),
    &get_handlebars(),
  )
  .unwrap();

  assert_that!(dots)
    .tap_mut(|d| d.has_length(3))
    .select_and(
      |d| d.iter().find(|d| d.0 == "/test03/test04").unwrap(),
      |d| {
        d.map(|d| &d.1).matches(|i| i.installs.as_ref().unwrap().cmd == "test04");
      },
    )
    .select_and(
      |d| d.iter().find(|d| d.0 == "/test03/test05").unwrap(),
      |d| {
        d.map(|d| &d.1).matches(|i| i.installs.as_ref().unwrap().cmd == "test03");
      },
    )
    .select_and(
      |d| d.iter().find(|d| d.0 == "/test03/test06").unwrap(),
      |d| {
        d.map(|d| &d.1).matches(|i| i.installs.as_ref().unwrap().cmd == "test03");
      },
    );
}

#[test]
fn read_all_file_formats() {
  let dots = read_dots(
    Path::new(file!()).parent().unwrap().join("data/file_formats").as_path(),
    &["/**".to_owned()],
    &Default::default(),
    &get_handlebars(),
  )
  .unwrap();

  assert_that!(dots).has_length(3);
  assert_that!(dots).mapped_contains(|d| &d.0, &"/test01");
  assert_that!(dots).mapped_contains(|d| &d.0, &"/test02");
  assert_that!(dots).mapped_contains(|d| &d.0, &"/test03");
}
