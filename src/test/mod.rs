use std::path::Path;

use rstest::rstest;
use speculoos::assert_that;
use tap::Conv;

use crate::cli::Cli;

#[rstest]
#[case("dotfiles01", "toml")]
#[case("dotfiles02", "json")]
#[case("dotfiles03", "force")]
fn read_config_formats(#[case] dotfiles_path: &str, #[case] expexted: &str) {
  let mut cli = Cli {
    dry_run: true,
    command: crate::cli::Command::Init { repo: None },
    config: Path::new(file!()).parent().unwrap().join("data/config/config.yaml").conv(),
    dotfiles: Some(Path::new(file!()).parent().unwrap().join("data").conv()),
  };

  let config = super::read_config(&cli).unwrap();

  assert_that!(config.variables["test01"]).is_equal_to(&"yaml".conv());

  cli.dotfiles = Some(Path::new(file!()).parent().unwrap().join("data").join(dotfiles_path).conv());

  let config = super::read_config(&cli).unwrap();

  assert_that!(config.variables["test01"]).is_equal_to(&"yaml".conv());
  assert_that!(config.variables["test02"]).is_equal_to(&expexted.conv());
}
