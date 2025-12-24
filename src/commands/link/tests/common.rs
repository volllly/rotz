use super::super::*;
use std::fs;
use tempfile::TempDir;

// ===== INTEGRATION TESTS FOR create_link FUNCTION =====

#[test]
fn test_create_link_source_does_not_exist() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("nonexistent.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  let result = create_link(&src_file, &dst_file, &LinkType::Copy, false, None);
  assert!(result.is_err());

  // Should be LinkSourceDoesNotExist error
  match result {
    Err(Error::LinkSourceDoesNotExist(path)) => {
      assert_eq!(path, src_file);
    }
    _ => panic!("Expected LinkSourceDoesNotExist error"),
  }
}

#[test]
fn test_create_link_already_exists_with_force() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  fs::write(&src_file, "source content").unwrap();
  fs::write(&dst_file, "existing content").unwrap();

  let result = create_link(&src_file, &dst_file, &LinkType::Copy, true, None);
  assert!(result.is_ok());

  assert!(dst_file.exists());
  assert_eq!(fs::read_to_string(&dst_file).unwrap(), "source content");
}

#[test]
fn test_create_link_with_linked_tracking_allows_overwrite() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  fs::write(&src_file, "source content").unwrap();
  fs::write(&dst_file, "existing content").unwrap();

  // Create a linked tracking map that contains this destination
  let mut linked_map = std::collections::HashMap::new();
  linked_map.insert(dst_file.clone(), temp_dir.path().join("old_source.txt"));

  let result = create_link(&src_file, &dst_file, &LinkType::Symbolic, false, Some(&linked_map));
  assert!(result.is_ok());

  assert!(dst_file.exists());
  assert!(dst_file.is_symlink());
  assert_eq!(fs::read_to_string(&dst_file).unwrap(), "source content");
}

#[test]
fn test_create_link_directory_with_force() {
  let temp_dir = TempDir::new().unwrap();
  let src_dir = temp_dir.path().join("source_dir");
  let dst_dir = temp_dir.path().join("dest_dir");

  // Create source directory
  fs::create_dir(&src_dir).unwrap();
  fs::write(src_dir.join("source_file.txt"), "source content").unwrap();

  // Create destination directory with different content
  fs::create_dir(&dst_dir).unwrap();
  fs::write(dst_dir.join("existing_file.txt"), "existing content").unwrap();

  let result = create_link(&src_dir, &dst_dir, &LinkType::Copy, true, None);
  assert!(result.is_ok());

  assert!(dst_dir.exists());
  assert!(dst_dir.join("source_file.txt").exists());
  // Note: copy operations merge directories, so existing files remain unless overwritten
  assert!(dst_dir.join("existing_file.txt").exists());
  assert_eq!(fs::read_to_string(dst_dir.join("source_file.txt")).unwrap(), "source content");
  assert_eq!(fs::read_to_string(dst_dir.join("existing_file.txt")).unwrap(), "existing content");
}

// ===== CROSS-PLATFORM COMPATIBILITY TESTS =====

#[test]
fn test_all_link_types_work_with_basic_files() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  fs::write(&src_file, "test content").unwrap();

  // Test Copy
  let copy_dst = temp_dir.path().join("copy_dest.txt");
  let copy_result = create_link(&src_file, &copy_dst, &LinkType::Copy, false, None);
  assert!(copy_result.is_ok());
  assert!(copy_dst.exists());
  assert!(!copy_dst.is_symlink());

  // Test Symbolic
  let sym_dst = temp_dir.path().join("sym_dest.txt");
  let sym_result = create_link(&src_file, &sym_dst, &LinkType::Symbolic, false, None);
  assert!(sym_result.is_ok());
  assert!(sym_dst.exists());
  assert!(sym_dst.is_symlink());

  // Test Hard
  let hard_dst = temp_dir.path().join("hard_dest.txt");
  let hard_result = create_link(&src_file, &hard_dst, &LinkType::Hard, false, None);
  assert!(hard_result.is_ok());
  assert!(hard_dst.exists());
  assert!(!hard_dst.is_symlink());

  // All should have the same content
  assert_eq!(fs::read_to_string(&copy_dst).unwrap(), "test content");
  assert_eq!(fs::read_to_string(&sym_dst).unwrap(), "test content");
  assert_eq!(fs::read_to_string(&hard_dst).unwrap(), "test content");
}

#[test]
fn test_parent_directory_creation_consistency() {
  let temp_dir = TempDir::new().unwrap();
  let src_file = temp_dir.path().join("source.txt");
  let nested_path = temp_dir.path().join("level1").join("level2").join("level3");

  fs::write(&src_file, "nested test").unwrap();

  // Test all link types create parent directories
  let copy_dst = nested_path.join("copy.txt");
  let sym_dst = nested_path.join("sym.txt");
  let hard_dst = nested_path.join("hard.txt");

  assert!(create_link(&src_file, &copy_dst, &LinkType::Copy, false, None).is_ok());
  assert!(create_link(&src_file, &sym_dst, &LinkType::Symbolic, false, None).is_ok());
  assert!(create_link(&src_file, &hard_dst, &LinkType::Hard, false, None).is_ok());

  assert!(copy_dst.exists());
  assert!(sym_dst.exists());
  assert!(hard_dst.exists());
}

// ===== ERROR HANDLING CONSISTENCY TESTS =====

#[test]
fn test_error_handling_consistency_across_link_types() {
  let temp_dir = TempDir::new().unwrap();
  let nonexistent_src = temp_dir.path().join("does_not_exist.txt");
  let dst_file = temp_dir.path().join("dest.txt");

  // All link types should fail consistently for nonexistent source
  let copy_result = create_link(&nonexistent_src, &dst_file, &LinkType::Copy, false, None);
  let sym_result = create_link(&nonexistent_src, &dst_file, &LinkType::Symbolic, false, None);
  let hard_result = create_link(&nonexistent_src, &dst_file, &LinkType::Hard, false, None);

  assert!(copy_result.is_err());
  assert!(sym_result.is_err());
  assert!(hard_result.is_err());

  // All should be LinkSourceDoesNotExist errors
  match copy_result {
    Err(Error::LinkSourceDoesNotExist(_)) => {}
    _ => panic!("Expected LinkSourceDoesNotExist error for copy"),
  }
  match sym_result {
    Err(Error::LinkSourceDoesNotExist(_)) => {}
    _ => panic!("Expected LinkSourceDoesNotExist error for symlink"),
  }
  match hard_result {
    Err(Error::LinkSourceDoesNotExist(_)) => {}
    _ => panic!("Expected LinkSourceDoesNotExist error for hardlink"),
  }
}
