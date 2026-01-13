use super::*;
use figment::value::Dict;
use std::path::PathBuf;

#[test]
fn test_config_helper_methods() {
  let paths = vec![
    PathBuf::from("/home/user/.dotfiles/personal"),
    PathBuf::from("/home/user/.dotfiles/work"),
  ];
  let config = Config {
    dotfiles: DotfilesPath::Multiple(paths.clone()),
    link_type: LinkType::Symbolic,
    shell_command: None,
    variables: Dict::new(),
  };

  assert_eq!(config.dotfiles_paths(), paths.as_slice());
  assert_eq!(config.primary_dotfiles_path(), &paths[0]);
  assert!(config.is_multi_dotfiles());
  assert_eq!(config.dotfiles_count(), 2);
}

#[test]
fn test_config_default() {
  let config = Config::default();
  assert!(!config.is_multi_dotfiles());
  assert_eq!(config.dotfiles_count(), 1);
  assert_eq!(config.primary_dotfiles_path(), &USER_DIRS.home_dir().join(".dotfiles"));
}

#[test]
fn test_config_resolve_dotfiles_homes() {
  let mut config = Config {
    dotfiles: DotfilesPath::Multiple(vec![
      PathBuf::from("~/.dotfiles/personal"),
      PathBuf::from("~/.dotfiles/work"),
    ]),
    link_type: LinkType::Symbolic,
    shell_command: None,
    variables: Dict::new(),
  };

  config.resolve_dotfiles_homes();

  // Paths should be resolved (though we can't test the exact result without knowing home dir)
  for path in config.dotfiles_paths() {
    assert!(!path.to_string_lossy().starts_with("~"));
  }
}

#[test]
fn test_config_single_dotfiles_methods() {
  let path = PathBuf::from("/home/user/.dotfiles");
  let config = Config {
    dotfiles: DotfilesPath::Single(path.clone()),
    link_type: LinkType::Symbolic,
    shell_command: None,
    variables: Dict::new(),
  };

  assert_eq!(config.dotfiles_paths(), &[path.clone()]);
  assert_eq!(config.primary_dotfiles_path(), &path);
  assert!(!config.is_multi_dotfiles());
  assert_eq!(config.dotfiles_count(), 1);
}

#[test]
fn test_config_resolve_single_home() {
  let mut config = Config {
    dotfiles: DotfilesPath::Single(PathBuf::from("~/.dotfiles")),
    link_type: LinkType::Symbolic,
    shell_command: None,
    variables: Dict::new(),
  };

  config.resolve_dotfiles_homes();

  // Path should be resolved
  assert!(!config.primary_dotfiles_path().to_string_lossy().starts_with("~"));
}

#[test]
fn test_config_with_shell_command() {
  let config = Config {
    dotfiles: DotfilesPath::Single(PathBuf::from("/test")),
    link_type: LinkType::Hard,
    shell_command: Some("custom shell command".to_string()),
    variables: Dict::new(),
  };

  assert_eq!(config.shell_command, Some("custom shell command".to_string()));
  assert_eq!(config.link_type, LinkType::Hard);
}

#[test]
fn test_config_with_variables() {
  let mut variables = Dict::new();
  variables.insert("test_var".to_string(), "test_value".into());
  variables.insert("another_var".to_string(), 42.into());

  let config = Config {
    dotfiles: DotfilesPath::Single(PathBuf::from("/test")),
    link_type: LinkType::Symbolic,
    shell_command: None,
    variables: variables.clone(),
  };

  assert_eq!(config.variables, variables);
  assert_eq!(config.variables.len(), 2);
}
