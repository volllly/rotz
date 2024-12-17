#![allow(clippy::needless_borrows_for_generic_args)]
use std::path::Path;

use figment::{util::map, value};
use rstest::rstest;
use speculoos::prelude::*;

use super::{Engine, Parameters};
use crate::{
  cli::{Cli, Command, PathBuf},
  config::{Config, LinkType},
  dot::read_dots,
  helpers::os,
};

pub(crate) fn get_handlebars<'a>() -> Engine<'a> {
  let cli = Cli {
    dry_run: true,
    dotfiles: None,
    config: PathBuf("".into()),
    command: Command::Clone { repo: String::new() },
  };

  Engine::new(&Config::default(), &cli)
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
    command: Command::Clone { repo: String::new() },
  };

  assert_that!(Engine::new(&config, &cli).render(template, &Parameters { config: &config, name: "name" }).unwrap()).is_equal_to(expected.to_owned());
}

#[test]
fn name() {
  let dots = read_dots(
    Path::new(file!()).parent().unwrap().join("data/dotfiles01").as_path(),
    &["/**".to_owned()],
    &Default::default(),
    &get_handlebars(),
  )
  .unwrap();
  dbg!(&dots);
  assert_that!(dots.iter().find(|d| d.0 == "/test01/test02"))
    .is_some()
    .map(|d| &d.1.installs)
    .is_some()
    .map(|i| &i.cmd)
    .is_equal_to(&"test02".into());
}

#[test]
fn os_helpers() {
  let config = Config::default();

  assert_that!(get_handlebars()
    .render(
      "{{ #windows }}windows{{ /windows }}{{ #linux }}linux{{ /linux }}{{ #darwin }}darwin{{ /darwin }}",
      &Parameters { config: &config, name: "" }
    )
    .unwrap())
  .is_equal_to(os::OS.to_string().to_ascii_lowercase());
}

#[test]
fn os_else_helpers() {
  let config = Config::default();

  let mut expected = String::new();
  if !os::OS.is_windows() {
    expected += "else_windows";
  }
  if !os::OS.is_linux() {
    expected += "else_linux";
  }
  if !os::OS.is_darwin() {
    expected += "else_darwin";
  }
  assert_that!(get_handlebars()
    .render(
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
    command: Command::Clone { repo: String::new() },
  };

  assert_that!(Engine::new(&config, &cli).render("{{ eval \"echo 'test'\" }}", &Parameters { config: &config, name: "" }).unwrap()).is_equal_to("test".to_owned());
}
