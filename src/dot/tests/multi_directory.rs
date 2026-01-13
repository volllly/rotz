use super::*;

#[test]
fn test_read_dots_multi_directories() {
  let dir1 = Path::new(file!()).parent().unwrap().join("../test/data/directory_structure");
  let dir2 = Path::new(file!()).parent().unwrap().join("../test/data/file_formats");

  let dots = read_dots_multi(&[dir1.to_path_buf(), dir2.to_path_buf()], &["/**".to_owned()], &Default::default(), &get_handlebars()).unwrap();

  // Should get dots from both directories
  assert!(dots.len() >= 3);

  // Test conflict resolution - first directory should win for duplicates
  let test01_dots: Vec<_> = dots.iter().filter(|(name, _)| name == "/test01").collect();
  assert_eq!(test01_dots.len(), 1); // Only one /test01 despite being in both dirs
}

#[test]
fn test_read_dots_with_sources_tracking() {
  use crate::config::{Config, DotfilesPath, LinkType};

  let dir1 = Path::new(file!()).parent().unwrap().join("../test/data/directory_structure");
  let dir2 = Path::new(file!()).parent().unwrap().join("../test/data/file_formats");

  let config = Config {
    dotfiles: DotfilesPath::Multiple(vec![dir1.to_path_buf(), dir2.to_path_buf()]),
    link_type: LinkType::Symbolic,
    shell_command: None,
    variables: figment::value::Dict::new(),
  };

  let dots = read_dots_with_sources(&["/**".to_owned()], &config, &get_handlebars()).unwrap();

  // Should track source directory for each dot
  assert!(dots.len() >= 3);

  // Each dot should have a source directory
  for (_name, _dot, source) in &dots {
    assert!(source.exists());
    // Source should be one of our test directories
    assert!(source == &dir1 || source == &dir2);
  }
}

#[test]
fn test_read_dots_multi_empty_directories() {
  let dots = read_dots_multi(&[], &["/**".to_owned()], &Default::default(), &get_handlebars()).unwrap();

  assert!(dots.is_empty());
}

#[test]
fn test_read_dots_multi_nonexistent_directories() {
  let nonexistent = Path::new("/nonexistent/directory");
  let existing = Path::new(file!()).parent().unwrap().join("../test/data/directory_structure");

  let dots = read_dots_multi(&[nonexistent.to_path_buf(), existing.to_path_buf()], &["/**".to_owned()], &Default::default(), &get_handlebars()).unwrap();

  // Should still read dots from the existing directory
  assert!(dots.len() >= 3);
}

#[test]
fn test_read_dots_from_config() {
  use crate::config::{Config, DotfilesPath, LinkType};

  let dir1 = Path::new(file!()).parent().unwrap().join("../test/data/directory_structure");

  let config = Config {
    dotfiles: DotfilesPath::Single(dir1.to_path_buf()),
    link_type: LinkType::Symbolic,
    shell_command: None,
    variables: figment::value::Dict::new(),
  };

  let dots = read_dots_from_config(&["/**".to_owned()], &config, &get_handlebars()).unwrap();

  // Should read dots from the single directory
  assert!(dots.len() >= 3);
}

#[test]
fn test_read_dots_multi_conflict_resolution() {
  // Create a scenario where the same dot exists in multiple directories
  let dir1 = Path::new(file!()).parent().unwrap().join("../test/data/directory_structure");
  let dir2 = Path::new(file!()).parent().unwrap().join("../test/data/file_formats");

  let dots = read_dots_multi(&[dir1.to_path_buf(), dir2.to_path_buf()], &["/test01".to_owned()], &Default::default(), &get_handlebars()).unwrap();

  // Should only get one dot even if it exists in both directories
  let test01_count = dots.iter().filter(|(name, _)| name == "/test01").count();
  assert_eq!(test01_count, 1);
}

#[test]
fn test_read_dots_with_sources_conflict_resolution() {
  use crate::config::{Config, DotfilesPath, LinkType};

  let dir1 = Path::new(file!()).parent().unwrap().join("../test/data/directory_structure");
  let dir2 = Path::new(file!()).parent().unwrap().join("../test/data/file_formats");

  let config = Config {
    dotfiles: DotfilesPath::Multiple(vec![dir1.to_path_buf(), dir2.to_path_buf()]),
    link_type: LinkType::Symbolic,
    shell_command: None,
    variables: figment::value::Dict::new(),
  };

  let dots = read_dots_with_sources(&["/test01".to_owned()], &config, &get_handlebars()).unwrap();

  // Should only get one dot and it should be from the first directory
  let test01_dots: Vec<_> = dots.iter().filter(|(name, _, _)| name == "/test01").collect();
  assert_eq!(test01_dots.len(), 1);

  if let Some((_, _, source)) = test01_dots.first() {
    assert_eq!(*source, dir1);
  }
}

#[test]
fn test_read_dots_multi_with_globs() {
  let dir1 = Path::new(file!()).parent().unwrap().join("../test/data/directory_structure");

  // Test with specific glob patterns
  let dots = read_dots_multi(&[dir1.to_path_buf()], &["/test03/*".to_owned()], &Default::default(), &get_handlebars()).unwrap();

  // Should only get dots matching the glob
  for (name, _) in &dots {
    assert!(name.starts_with("/test03/"));
  }
}

#[test]
fn test_read_dots_multi_priority_ordering() {
  let dir1 = Path::new(file!()).parent().unwrap().join("../test/data/directory_structure");
  let dir2 = Path::new(file!()).parent().unwrap().join("../test/data/file_formats");

  // Test that order matters - dir1 first should win
  let dots1 = read_dots_multi(&[dir1.to_path_buf(), dir2.to_path_buf()], &["/test01".to_owned()], &Default::default(), &get_handlebars()).unwrap();

  // Test that order matters - dir2 first should win
  let dots2 = read_dots_multi(&[dir2.to_path_buf(), dir1.to_path_buf()], &["/test01".to_owned()], &Default::default(), &get_handlebars()).unwrap();

  // Both should have exactly one result but potentially from different sources
  assert_eq!(dots1.len(), 1);
  assert_eq!(dots2.len(), 1);
}
