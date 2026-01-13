use super::*;
use serde_json;
use std::path::PathBuf;

#[test]
fn test_serde_single_path_string() {
  let json = r#"{"dotfiles": "/home/user/.dotfiles"}"#;
  let config: serde_json::Result<serde_json::Value> = serde_json::from_str(json);
  assert!(config.is_ok());

  // Test that we can deserialize a single path as string
  let dotfiles_path: DotfilesPath = serde_json::from_value(serde_json::Value::String("/home/user/.dotfiles".to_string())).unwrap();

  assert!(!dotfiles_path.is_multiple());
  assert_eq!(dotfiles_path.first_path(), &PathBuf::from("/home/user/.dotfiles"));
}

#[test]
fn test_serde_multiple_paths_array() {
  let json_value = serde_json::json!(["/home/user/.dotfiles/personal", "/home/user/.dotfiles/work"]);

  let dotfiles_path: DotfilesPath = serde_json::from_value(json_value).unwrap();

  assert!(dotfiles_path.is_multiple());
  assert_eq!(dotfiles_path.len(), 2);
  assert_eq!(dotfiles_path.first_path(), &PathBuf::from("/home/user/.dotfiles/personal"));
}

#[test]
fn test_serde_single_path_in_array() {
  let json_value = serde_json::json!(["/home/user/.dotfiles"]);

  let dotfiles_path: DotfilesPath = serde_json::from_value(json_value).unwrap();

  // Should convert to Single variant when array has only one element
  assert!(!dotfiles_path.is_multiple());
  assert_eq!(dotfiles_path.len(), 1);
}

#[test]
fn test_serde_serialization() {
  let single = DotfilesPath::Single(PathBuf::from("/home/user/.dotfiles"));
  let single_json = serde_json::to_value(&single).unwrap();
  assert_eq!(single_json, serde_json::Value::String("/home/user/.dotfiles".to_string()));

  let multiple = DotfilesPath::Multiple(vec![PathBuf::from("/home/user/.dotfiles/personal"), PathBuf::from("/home/user/.dotfiles/work")]);
  let multiple_json = serde_json::to_value(&multiple).unwrap();
  assert_eq!(multiple_json, serde_json::json!(["/home/user/.dotfiles/personal", "/home/user/.dotfiles/work"]));
}

#[test]
fn test_serde_validation_during_deserialization() {
  // Test that validation happens during deserialization
  let invalid_json = serde_json::json!([]);
  let result: Result<DotfilesPath, _> = serde_json::from_value(invalid_json);
  assert!(result.is_err());
}

#[test]
fn test_serde_yaml_single_path() {
  let yaml = "dotfiles: /home/user/.dotfiles";
  let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();

  if let serde_yaml::Value::Mapping(map) = value {
    if let Some(dotfiles_value) = map.get(&serde_yaml::Value::String("dotfiles".to_string())) {
      let dotfiles_path: DotfilesPath = serde_yaml::from_value(dotfiles_value.clone()).unwrap();
      assert!(!dotfiles_path.is_multiple());
      assert_eq!(dotfiles_path.first_path(), &PathBuf::from("/home/user/.dotfiles"));
    }
  }
}

#[test]
fn test_serde_yaml_multiple_paths() {
  let yaml = r#"
dotfiles:
  - /home/user/.dotfiles/personal
  - /home/user/.dotfiles/work
"#;
  let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();

  if let serde_yaml::Value::Mapping(map) = value {
    if let Some(dotfiles_value) = map.get(&serde_yaml::Value::String("dotfiles".to_string())) {
      let dotfiles_path: DotfilesPath = serde_yaml::from_value(dotfiles_value.clone()).unwrap();
      assert!(dotfiles_path.is_multiple());
      assert_eq!(dotfiles_path.len(), 2);
    }
  }
}

#[test]
fn test_serde_toml_single_path() {
  use figment::Figment;
  use figment::providers::Serialized;

  let toml_data = std::collections::HashMap::from([("dotfiles".to_string(), "/home/user/.dotfiles".to_string())]);
  let figment = Figment::new().merge(Serialized::defaults(toml_data));
  let dotfiles_path: DotfilesPath = figment.extract_inner("dotfiles").unwrap();

  assert!(!dotfiles_path.is_multiple());
  assert_eq!(dotfiles_path.first_path(), &PathBuf::from("/home/user/.dotfiles"));
}

#[test]
fn test_serde_toml_multiple_paths() {
  use figment::Figment;
  use figment::providers::Serialized;

  let toml_data = std::collections::HashMap::from([("dotfiles".to_string(), vec!["/home/user/.dotfiles/personal".to_string(), "/home/user/.dotfiles/work".to_string()])]);
  let figment = Figment::new().merge(Serialized::defaults(toml_data));
  let dotfiles_path: DotfilesPath = figment.extract_inner("dotfiles").unwrap();

  assert!(dotfiles_path.is_multiple());
  assert_eq!(dotfiles_path.len(), 2);
}
