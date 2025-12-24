use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_symlink_single_file() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  fs::write(&src_file, "symlink test content").unwrap();

  symlink(&src_file, &dst_file).unwrap();

  assert!(dst_file.exists());
  assert!(dst_file.is_symlink());
  assert_eq!(fs::read_to_string(&dst_file).unwrap(), "symlink test content");

  // Verify it's actually a symlink by checking the link target
  #[cfg(unix)]
  {
    let link_target = fs::read_link(&dst_file).unwrap();
    assert_eq!(link_target, src_file);
  }

  #[cfg(windows)]
  {
    let metadata = fs::symlink_metadata(&dst_file).unwrap();
    assert!(metadata.is_symlink());
  }
}

#[test]
fn test_symlink_directory() {
  let temp_dir = TempDir::new().unwrap();
  let src_dir = temp_dir.path().join("source_dir");
  let dst_dir = temp_dir.path().join("dest_dir");

  fs::create_dir(&src_dir).unwrap();
  fs::write(src_dir.join("file1.txt"), "content1").unwrap();
  fs::write(src_dir.join("file2.txt"), "content2").unwrap();

  symlink(&src_dir, &dst_dir).unwrap();

  assert!(dst_dir.exists());
  assert!(dst_dir.is_dir());
  assert!(dst_dir.is_symlink());

  // Should be able to access files through the symlink
  assert!(dst_dir.join("file1.txt").exists());
  assert!(dst_dir.join("file2.txt").exists());
  assert_eq!(fs::read_to_string(dst_dir.join("file1.txt")).unwrap(), "content1");
  assert_eq!(fs::read_to_string(dst_dir.join("file2.txt")).unwrap(), "content2");
}

#[test]
fn test_symlink_creates_parent_directories() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("nested").join("deeply").join("nested").join("dest.txt");

  fs::write(&src_file, "symlink nested content").unwrap();

  symlink(&src_file, &dst_file).unwrap();

  assert!(dst_file.exists());
  assert!(dst_file.is_symlink());
  assert_eq!(fs::read_to_string(&dst_file).unwrap(), "symlink nested content");
}

#[test]
fn test_symlink_broken_when_source_removed() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  fs::write(&src_file, "temporary content").unwrap();
  symlink(&src_file, &dst_file).unwrap();

  assert!(dst_file.exists());

  // Remove source file
  fs::remove_file(&src_file).unwrap();

  // Symlink should still exist but be broken
  assert!(dst_file.is_symlink());
  // Reading should fail because the target doesn't exist
  assert!(fs::read_to_string(&dst_file).is_err());
}

#[test]
fn test_create_link_with_symbolic_type() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  fs::write(&src_file, "symbolic integration test").unwrap();

  let result = create_link(&src_file, &dst_file, &LinkType::Symbolic, false, None);
  assert!(result.is_ok());

  assert!(dst_file.exists());
  assert!(dst_file.is_symlink());
  assert_eq!(fs::read_to_string(&dst_file).unwrap(), "symbolic integration test");
}

#[test]
fn test_create_link_symlink_fails_on_existing_without_force() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  fs::write(&src_file, "source content").unwrap();
  fs::write(&dst_file, "existing content").unwrap();

  let result = create_link(&src_file, &dst_file, &LinkType::Symbolic, false, None);
  assert!(result.is_err());

  match result {
    Err(Error::AlreadyExists(path)) => {
      assert_eq!(path, dst_file);
    }
    _ => panic!("Expected AlreadyExists error"),
  }
}
