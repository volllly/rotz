use std::path::Path;

use super::read_dots;
use speculoos::prelude::*;

#[test]
fn read_all_dots() {
  let dots = read_dots(
    std::path::Path::new(file!()).parent().unwrap().join("data/directory_structure").as_path(),
    &["**".to_string()],
    &Default::default(),
  )
  .unwrap();

  assert_that!(dots).has_length(4);
  assert_that!(dots).mapped_contains(|d| d.0.as_path(), &Path::new("test01"));
  assert_that!(dots).mapped_contains(|d| d.0.as_path(), &Path::new("test02"));
  assert_that!(dots).mapped_contains(|d| d.0.as_path(), &Path::new("test03/test04"));
  assert_that!(dots).mapped_contains(|d| d.0.as_path(), &Path::new("test03/test05"));
}

#[test]
fn read_sub_dots() {
  let dots = read_dots(
    std::path::Path::new(file!()).parent().unwrap().join("data/directory_structure").as_path(),
    &["test03/*".to_string()],
    &Default::default(),
  )
  .unwrap();

  assert_that!(dots).has_length(2);
  assert_that!(dots).mapped_contains(|d| d.0.as_path(), &Path::new("test03/test04"));
  assert_that!(dots).mapped_contains(|d| d.0.as_path(), &Path::new("test03/test05"));
}

#[test]
fn read_non_sub_dots() {
  let dots = read_dots(
    std::path::Path::new(file!()).parent().unwrap().join("data/directory_structure").as_path(),
    &["*".to_string()],
    &Default::default(),
  )
  .unwrap();

  assert_that!(dots).has_length(2);
  assert_that!(dots).mapped_contains(|d| d.0.as_path(), &Path::new("test01"));
  assert_that!(dots).mapped_contains(|d| d.0.as_path(), &Path::new("test02"));
}
