use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_hardlink_single_file() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  fs::write(&src_file, "hardlink test content").unwrap();

  hardlink(&src_file, &dst_file).unwrap();

  assert!(dst_file.exists());
  assert!(!dst_file.is_symlink()); // Hard links are not symlinks
  assert_eq!(fs::read_to_string(&dst_file).unwrap(), "hardlink test content");

  // Verify they're actually the same file (same inode on Unix)
  #[cfg(unix)]
  {
    use std::os::unix::fs::MetadataExt;
    let src_metadata = fs::metadata(&src_file).unwrap();
    let dst_metadata = fs::metadata(&dst_file).unwrap();
    assert_eq!(src_metadata.ino(), dst_metadata.ino());
    assert_eq!(src_metadata.nlink(), 2); // Should have 2 hard links now
  }

  // On Windows, verify they both exist and have same content
  #[cfg(windows)]
  {
    assert_eq!(fs::read_to_string(&src_file).unwrap(), fs::read_to_string(&dst_file).unwrap());
  }
}

#[test]
fn test_hardlink_persists_when_source_removed() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  fs::write(&src_file, "persistent content").unwrap();
  hardlink(&src_file, &dst_file).unwrap();

  assert!(dst_file.exists());
  assert_eq!(fs::read_to_string(&dst_file).unwrap(), "persistent content");

  // Remove source file
  fs::remove_file(&src_file).unwrap();

  // Hard link should still exist and be readable
  assert!(dst_file.exists());
  assert_eq!(fs::read_to_string(&dst_file).unwrap(), "persistent content");
}

#[test]
fn test_hardlink_creates_parent_directories() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("nested").join("deeply").join("nested").join("dest.txt");

  fs::write(&src_file, "hardlink nested content").unwrap();

  hardlink(&src_file, &dst_file).unwrap();

  assert!(dst_file.exists());
  assert!(!dst_file.is_symlink());
  assert_eq!(fs::read_to_string(&dst_file).unwrap(), "hardlink nested content");
}

#[test]
#[cfg(unix)] // Hard linking directories is typically not allowed on most filesystems
fn test_hardlink_directory_on_unix_fails() {
  let temp_dir = TempDir::new().unwrap();
  let src_dir = temp_dir.path().join("source_dir");
  let dst_dir = temp_dir.path().join("dest_dir");

  fs::create_dir(&src_dir).unwrap();
  fs::write(src_dir.join("file.txt"), "content").unwrap();

  // Hard linking directories should typically fail on Unix systems
  let result = hardlink(&src_dir, &dst_dir);
  assert!(result.is_err());
}

#[test]
#[cfg(windows)] // On Windows, hardlink uses junction for directories
fn test_hardlink_directory_on_windows() {
  let temp_dir = TempDir::new().unwrap();
  let src_dir = temp_dir.path().join("source_dir");
  let dst_dir = temp_dir.path().join("dest_dir");

  fs::create_dir(&src_dir).unwrap();
  fs::write(src_dir.join("file.txt"), "content").unwrap();

  hardlink(&src_dir, &dst_dir).unwrap();

  assert!(dst_dir.exists());
  assert!(dst_dir.is_dir());
  assert!(dst_dir.join("file.txt").exists());
  assert_eq!(fs::read_to_string(dst_dir.join("file.txt")).unwrap(), "content");
}

#[test]
fn test_create_link_with_hard_type() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  fs::write(&src_file, "hard integration test").unwrap();

  let result = create_link(&src_file, &dst_file, &LinkType::Hard, false, None);
  assert!(result.is_ok());

  assert!(dst_file.exists());
  assert!(!dst_file.is_symlink());
  assert_eq!(fs::read_to_string(&dst_file).unwrap(), "hard integration test");
}

#[test]
fn test_create_link_hardlink_fails_on_existing_without_force() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  fs::write(&src_file, "source content").unwrap();
  fs::write(&dst_file, "existing content").unwrap();

  let result = create_link(&src_file, &dst_file, &LinkType::Hard, false, None);
  assert!(result.is_err());

  match result {
    Err(Error::AlreadyExists(path)) => {
      assert_eq!(path, dst_file);
    }
    _ => panic!("Expected AlreadyExists error"),
  }
}
