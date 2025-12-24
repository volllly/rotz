use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_copyfile_single_file() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  fs::write(&src_file, "test content").unwrap();

  copyfile(&src_file, &dst_file).unwrap();

  assert!(dst_file.exists());
  assert_eq!(fs::read_to_string(&dst_file).unwrap(), "test content");
}

#[test]
fn test_copyfile_directory() {
  let temp_dir = TempDir::new().unwrap();
  let src_dir = temp_dir.path().join("source_dir");
  let dst_dir = temp_dir.path().join("dest_dir");

  fs::create_dir(&src_dir).unwrap();
  fs::write(src_dir.join("file1.txt"), "content1").unwrap();
  fs::write(src_dir.join("file2.txt"), "content2").unwrap();

  copyfile(&src_dir, &dst_dir).unwrap();

  assert!(dst_dir.exists());
  assert!(dst_dir.join("file1.txt").exists());
  assert!(dst_dir.join("file2.txt").exists());
  assert_eq!(fs::read_to_string(dst_dir.join("file1.txt")).unwrap(), "content1");
  assert_eq!(fs::read_to_string(dst_dir.join("file2.txt")).unwrap(), "content2");
}

#[test]
fn test_copy_dir_all_nested() {
  let temp_dir = TempDir::new().unwrap();
  let src_dir = temp_dir.path().join("source");
  let dst_dir = temp_dir.path().join("dest");
  let nested_dir = src_dir.join("nested");

  fs::create_dir_all(&nested_dir).unwrap();
  fs::write(src_dir.join("root.txt"), "root content").unwrap();
  fs::write(nested_dir.join("nested.txt"), "nested content").unwrap();

  copy_dir_all(&src_dir, &dst_dir).unwrap();

  assert!(dst_dir.exists());
  assert!(dst_dir.join("root.txt").exists());
  assert!(dst_dir.join("nested").exists());
  assert!(dst_dir.join("nested/nested.txt").exists());
  assert_eq!(fs::read_to_string(dst_dir.join("root.txt")).unwrap(), "root content");
  assert_eq!(fs::read_to_string(dst_dir.join("nested/nested.txt")).unwrap(), "nested content");
}

#[test]
fn test_copyfile_creates_parent_directories() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("nested").join("deeply").join("nested").join("dest.txt");

  fs::write(&src_file, "test content").unwrap();

  copyfile(&src_file, &dst_file).unwrap();

  assert!(dst_file.exists());
  assert_eq!(fs::read_to_string(&dst_file).unwrap(), "test content");
}

#[test]
fn test_copyfile_empty_directory() {
  let temp_dir = TempDir::new().unwrap();
  let src_dir = temp_dir.path().join("empty_source");
  let dst_dir = temp_dir.path().join("empty_dest");

  fs::create_dir(&src_dir).unwrap();

  copyfile(&src_dir, &dst_dir).unwrap();

  assert!(dst_dir.exists());
  assert!(dst_dir.is_dir());
  assert_eq!(fs::read_dir(&dst_dir).unwrap().count(), 0);
}

#[test]
fn test_copy_dir_all_preserves_structure() {
  let temp_dir = TempDir::new().unwrap();
  let src_dir = temp_dir.path().join("complex_source");
  let dst_dir = temp_dir.path().join("complex_dest");

  // Create a complex directory structure
  let subdir1 = src_dir.join("subdir1");
  let subdir2 = src_dir.join("subdir2");
  let nested = subdir1.join("nested");

  fs::create_dir_all(&nested).unwrap();
  fs::create_dir_all(&subdir2).unwrap();

  fs::write(src_dir.join("root.txt"), "root").unwrap();
  fs::write(subdir1.join("sub1.txt"), "sub1").unwrap();
  fs::write(subdir2.join("sub2.txt"), "sub2").unwrap();
  fs::write(nested.join("nested.txt"), "nested").unwrap();

  copy_dir_all(&src_dir, &dst_dir).unwrap();

  // Verify structure is preserved
  assert!(dst_dir.join("root.txt").exists());
  assert!(dst_dir.join("subdir1").join("sub1.txt").exists());
  assert!(dst_dir.join("subdir2").join("sub2.txt").exists());
  assert!(dst_dir.join("subdir1").join("nested").join("nested.txt").exists());

  // Verify content is preserved
  assert_eq!(fs::read_to_string(dst_dir.join("root.txt")).unwrap(), "root");
  assert_eq!(fs::read_to_string(dst_dir.join("subdir1").join("sub1.txt")).unwrap(), "sub1");
  assert_eq!(fs::read_to_string(dst_dir.join("subdir2").join("sub2.txt")).unwrap(), "sub2");
  assert_eq!(fs::read_to_string(dst_dir.join("subdir1").join("nested").join("nested.txt")).unwrap(), "nested");
}

#[test]
fn test_create_link_with_copy_type() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  fs::write(&src_file, "integration test").unwrap();

  let result = create_link(&src_file, &dst_file, &LinkType::Copy, false, None);
  assert!(result.is_ok());

  assert!(dst_file.exists());
  assert!(!dst_file.is_symlink());
  assert_eq!(fs::read_to_string(&dst_file).unwrap(), "integration test");
}

#[test]
fn test_create_link_copy_overwrites_existing() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  fs::write(&src_file, "source content").unwrap();
  fs::write(&dst_file, "existing content").unwrap();

  // Copy type always overwrites existing files
  let result = create_link(&src_file, &dst_file, &LinkType::Copy, false, None);
  assert!(result.is_ok());
  assert_eq!(fs::read_to_string(&dst_file).unwrap(), "source content");
}
