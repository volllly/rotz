use super::*;

use std::path::PathBuf;

#[test]
fn test_dotfiles_path_single_creation() {
  let path = PathBuf::from("/home/user/.dotfiles");
  let dotfiles_path = DotfilesPath::from(path.clone());

  assert_eq!(dotfiles_path.paths(), &[path.clone()]);
  assert_eq!(dotfiles_path.first_path(), &path);
  assert!(!dotfiles_path.is_multiple());
  assert_eq!(dotfiles_path.len(), 1);
}

#[test]
fn test_dotfiles_path_multiple_creation() {
  let paths = vec![PathBuf::from("/home/user/.dotfiles/personal"), PathBuf::from("/home/user/.dotfiles/work")];
  let dotfiles_path = DotfilesPath::from(paths.clone());

  assert_eq!(dotfiles_path.paths(), paths.as_slice());
  assert_eq!(dotfiles_path.first_path(), &paths[0]);
  assert!(dotfiles_path.is_multiple());
  assert_eq!(dotfiles_path.len(), 2);
}

#[test]
fn test_dotfiles_path_single_from_vec() {
  let paths = vec![PathBuf::from("/home/user/.dotfiles")];
  let dotfiles_path = DotfilesPath::from(paths);

  // Should convert to Single variant when only one path
  matches!(dotfiles_path, DotfilesPath::Single(_));
  assert!(!dotfiles_path.is_multiple());
}

#[test]
fn test_dotfiles_path_contains() {
  let path1 = PathBuf::from("/home/user/.dotfiles/personal");
  let path2 = PathBuf::from("/home/user/.dotfiles/work");
  let path3 = PathBuf::from("/home/user/.dotfiles/system");

  let dotfiles_path = DotfilesPath::Multiple(vec![path1.clone(), path2.clone()]);

  assert!(dotfiles_path.contains_path(&path1));
  assert!(dotfiles_path.contains_path(&path2));
  assert!(!dotfiles_path.contains_path(&path3));
}

#[test]
fn test_dotfiles_path_as_single() {
  let path = PathBuf::from("/home/user/.dotfiles");
  let single = DotfilesPath::Single(path.clone());
  let multiple = DotfilesPath::Multiple(vec![path.clone(), PathBuf::from("/other")]);

  assert_eq!(single.as_single(), Some(&path));
  assert_eq!(multiple.as_single(), None);
}

#[test]
fn test_dotfiles_path_string_methods() {
  let path = PathBuf::from("/home/user/.dotfiles");
  let dotfiles_path = DotfilesPath::Single(path.clone());

  assert_eq!(dotfiles_path.to_string_lossy(), path.to_string_lossy());
  assert_eq!(dotfiles_path.as_os_str(), path.as_os_str());
}

#[test]
fn test_dotfiles_path_join() {
  let path = PathBuf::from("/home/user/.dotfiles");
  let dotfiles_path = DotfilesPath::Single(path.clone());

  let joined = dotfiles_path.join("subfolder");
  assert_eq!(joined, path.join("subfolder"));
}

#[test]
fn test_dotfiles_path_as_ref() {
  let path = PathBuf::from("/home/user/.dotfiles");
  let dotfiles_path = DotfilesPath::Single(path.clone());

  let path_ref: &Path = dotfiles_path.as_ref();
  assert_eq!(path_ref, path.as_path());
}

#[test]
fn test_dotfiles_path_validation_empty() {
  let empty = DotfilesPath::Multiple(vec![]);
  assert!(matches!(empty.validate(), Err(DotfilesPathError::EmptyPaths)));
}

#[test]
fn test_dotfiles_path_validation_duplicates() {
  let path = PathBuf::from("/home/user/.dotfiles");
  let duplicate = DotfilesPath::Multiple(vec![path.clone(), path.clone()]);

  match duplicate.validate() {
    Err(DotfilesPathError::DuplicatePath(_)) => (),
    _ => panic!("Expected DuplicatePath error"),
  }
}

#[test]
fn test_dotfiles_path_validation_success() {
  let path1 = PathBuf::from("/home/user/.dotfiles/personal");
  let path2 = PathBuf::from("/home/user/.dotfiles/work");
  let valid = DotfilesPath::Multiple(vec![path1, path2]);

  assert!(valid.validate().is_ok());
}

#[test]
fn test_from_str_trait() {
  let dotfiles_path: DotfilesPath = "test_path".into();
  assert!(!dotfiles_path.is_multiple());
  assert_eq!(dotfiles_path.first_path(), &PathBuf::from("test_path"));
}
