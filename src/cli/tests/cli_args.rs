use super::*;

#[test]
fn test_empty_dotfiles_cli() {
  let cli = Cli {
    dotfiles: None,
    config: PathBuf(StdPathBuf::from("config.yaml")),
    dry_run: false,
    command: Command::Link {
      link: LinkRaw {
        dots: Default::default(),
        force: false,
        link_type: None,
      },
    },
  };

  let data = cli.data().unwrap();
  let global_dict = data.get(&Profile::Global).unwrap();

  // Should not contain dotfiles key when None
  assert!(!global_dict.contains_key("dotfiles"));
}

#[test]
fn test_single_dotfiles_cli() {
  let cli = Cli {
    dotfiles: Some(PathBuf(StdPathBuf::from("/home/user/.dotfiles"))),
    config: PathBuf(StdPathBuf::from("config.yaml")),
    dry_run: false,
    command: Command::Link {
      link: LinkRaw {
        dots: Default::default(),
        force: false,
        link_type: None,
      },
    },
  };

  let data = cli.data().unwrap();
  let global_dict = data.get(&Profile::Global).unwrap();

  // Should serialize as single string
  assert_eq!(global_dict.get("dotfiles").unwrap().as_str().unwrap(), "/home/user/.dotfiles");
}

#[test]
fn test_cli_dry_run_flag() {
  let cli = Cli {
    dotfiles: None,
    config: PathBuf(StdPathBuf::from("config.yaml")),
    dry_run: true,
    command: Command::Link {
      link: LinkRaw {
        dots: Default::default(),
        force: false,
        link_type: None,
      },
    },
  };

  assert!(cli.dry_run);
}

#[test]
fn test_cli_with_link_type() {
  let cli = Cli {
    dotfiles: None,
    config: PathBuf(StdPathBuf::from("config.yaml")),
    dry_run: false,
    command: Command::Link {
      link: LinkRaw {
        dots: Default::default(),
        force: false,
        link_type: Some(crate::config::LinkType::Hard),
      },
    },
  };

  let data = cli.data().unwrap();
  let global_dict = data.get(&Profile::Global).unwrap();

  // Should contain link_type when specified in command
  assert!(global_dict.contains_key("link_type"));
}

#[test]
fn test_cli_config_path() {
  let config_path = StdPathBuf::from("/custom/config/path.yaml");
  let cli = Cli {
    dotfiles: None,
    config: PathBuf(config_path.clone()),
    dry_run: false,
    command: Command::Link {
      link: LinkRaw {
        dots: Default::default(),
        force: false,
        link_type: None,
      },
    },
  };

  assert_eq!(cli.config.0, config_path);
}

#[test]
fn test_cli_different_commands() {
  let cli_clone = Cli {
    dotfiles: None,
    config: PathBuf(StdPathBuf::from("config.yaml")),
    dry_run: false,
    command: Command::Clone {
      repo: "https://github.com/user/dotfiles".to_string(),
    },
  };

  let cli_init = Cli {
    dotfiles: None,
    config: PathBuf(StdPathBuf::from("config.yaml")),
    dry_run: false,
    command: Command::Init { repo: Some("origin".to_string()) },
  };

  // Should not panic when creating different command types
  matches!(cli_clone.command, Command::Clone { .. });
  matches!(cli_init.command, Command::Init { .. });
}

#[test]
fn test_cli_pathbuf_wrapper() {
  let std_path = StdPathBuf::from("/test/path");
  let wrapped_path = PathBuf(std_path.clone());

  assert_eq!(wrapped_path.0, std_path);
}

#[test]
fn test_cli_pathbuf_from_str() {
  let path_str = "/test/path";
  let pathbuf = PathBuf::from_str(path_str).unwrap();

  assert_eq!(pathbuf.0, StdPathBuf::from(path_str));
}
