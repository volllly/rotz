use std::path::Path;

use speculoos::prelude::*;

use super::read_dots;

mod data;

#[test]
fn read_all_dots() {
  let dots = read_dots(Path::new(file!()).parent().unwrap().join("data/directory_structure").as_path(), &["**".to_owned()], &Default::default()).unwrap();

  assert_that!(dots).has_length(4);
  assert_that!(dots).mapped_contains(|d| &d.0, &"test01");
  assert_that!(dots).mapped_contains(|d| &d.0, &"test02");
  assert_that!(dots).mapped_contains(|d| &d.0, &"test03/test04");
  assert_that!(dots).mapped_contains(|d| &d.0, &"test03/test05");
}

#[test]
fn read_sub_dots() {
  let dots = read_dots(
    Path::new(file!()).parent().unwrap().join("data/directory_structure").as_path(),
    &["test03/*".to_owned()],
    &Default::default(),
  )
  .unwrap();

  assert_that!(dots).has_length(2);
  assert_that!(dots).mapped_contains(|d| &d.0, &"test03/test04");
  assert_that!(dots).mapped_contains(|d| &d.0, &"test03/test05");
}

#[test]
fn read_non_sub_dots() {
  let dots = read_dots(Path::new(file!()).parent().unwrap().join("data/directory_structure").as_path(), &["*".to_owned()], &Default::default()).unwrap();

  assert_that!(dots).has_length(2);
  assert_that!(dots).mapped_contains(|d| &d.0, &"test01");
  assert_that!(dots).mapped_contains(|d| &d.0, &"test02");
}

#[test]
fn read_all_file_formats() {
  let dots = read_dots(Path::new(file!()).parent().unwrap().join("data/file_formats").as_path(), &["**".to_owned()], &Default::default()).unwrap();

  assert_that!(dots).has_length(3);
  assert_that!(dots).mapped_contains(|d| &d.0, &"test01");
  assert_that!(dots).mapped_contains(|d| &d.0, &"test02");
  assert_that!(dots).mapped_contains(|d| &d.0, &"test03");
}
