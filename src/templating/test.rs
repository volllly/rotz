use figment::{util::map, value};
use rstest::rstest;
use speculoos::prelude::*;

use super::{render, Parameters};
use crate::{
  cli::{Cli, Command, PathBuf},
  config::{Config, LinkType},
  helpers::os,
};

pub fn init_handlebars() {
  let cli = Cli {
    dry_run: true,
    dotfiles: None,
    config: PathBuf("".into()),
    command: Command::Clone { repo: "".to_owned() },
  };

  crate::init_handlebars(&Config::default(), &cli).unwrap();
}

#[rstest]
#[case("{{ config.variables.test }}", "test")]
#[case("{{ config.variables.nested.nest }}", "nest")]
#[case("{{ whoami.username }}", &whoami::username())]
#[case("{{ dirs.user.home }}", &directories::UserDirs::new().unwrap().home_dir().to_string_lossy().to_string())]
#[case("{{ os }}", &crate::helpers::os::OS.to_string().to_ascii_lowercase())]
fn templating(#[case] template: &str, #[case] expected: &str) {
  let config = Config {
    dotfiles: "dotfiles".into(),
    link_type: LinkType::Hard,
    shell_command: "shell_command".to_owned().into(),
    variables: map! {
      "test".to_owned() => "test".into(),
      "nested".to_owned() => map!{
        "nest" => value::Value::from("nest")
      }.into()
    },
  };

  let cli = Cli {
    dry_run: true,
    dotfiles: None,
    config: PathBuf("".into()),
    command: Command::Clone { repo: "".to_owned() },
  };

  crate::init_handlebars(&config, &cli).unwrap();

  assert_that!(render(template, &Parameters { config: &config, name: "name" }).unwrap()).is_equal_to(expected.to_owned());
}

#[test]
fn os_helpers() {
  let config = Config::default();

  init_handlebars();

  assert_that!(render(
    "{{ #windows }}windows{{ /windows }}{{ #linux }}linux{{ /linux }}{{ #darwin }}darwin{{ /darwin }}",
    &Parameters { config: &config, name: "" }
  )
  .unwrap())
  .is_equal_to(os::OS.to_string().to_ascii_lowercase());
}

#[test]
fn os_else_helpers() {
  let config = Config::default();

  init_handlebars();

  let mut expected = "".to_owned();
  if !os::OS.is_windows() {
    expected += "else_windows";
  }
  if !os::OS.is_linux() {
    expected += "else_linux";
  }
  if !os::OS.is_darwin() {
    expected += "else_darwin";
  }
  assert_that!(render(
    "{{ #windows }}{{ else }}else_windows{{ /windows }}{{ #linux }}{{ else }}else_linux{{ /linux }}{{ #darwin }}{{ else }}else_darwin{{ /darwin }}",
    &Parameters { config: &config, name: "" }
  )
  .unwrap())
  .is_equal_to(expected);
}

#[test]
fn eval_helper() {
  let config = Config::default();

  let cli = Cli {
    dry_run: false,
    dotfiles: None,
    config: PathBuf("".into()),
    command: Command::Clone { repo: "".to_owned() },
  };

  crate::init_handlebars(&config, &cli).unwrap();

  assert_that!(render("{{ eval \"echo 'test'\" }}", &Parameters { config: &config, name: "" }).unwrap()).is_equal_to("test".to_owned());
}
