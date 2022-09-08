use rstest::rstest;
use serde_json::json;
use speculoos::assert_that;
use std::path::Path;

use crate::cli::Cli;

#[rstest]
#[case("dotfiles01", "toml")]
#[case("dotfiles02", "json")]
fn read_config_formats(#[case] dotfiles_path: &str, #[case] expexted: &str) {
  let mut cli = Cli {
    dry_run: true,
    command: crate::cli::Command::Init { repo: None },
    config: Path::new(file!()).parent().unwrap().join("data/config/config.yaml").into(),
    dotfiles: Some(Path::new(file!()).parent().unwrap().join("data").into()),
  };

  let config = super::read_config(&cli).unwrap();

  assert_that!(config.variables["test01"]).is_equal_to(json!("yaml"));

  cli.dotfiles = Some(Path::new(file!()).parent().unwrap().join("data").join(dotfiles_path).into());

  let config = super::read_config(&cli).unwrap();

  assert_that!(config.variables["test01"]).is_equal_to(json!("yaml"));
  assert_that!(config.variables["test02"]).is_equal_to(json!(expexted));
}
