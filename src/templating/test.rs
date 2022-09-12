use figment::{util::map, value};
use rstest::rstest;
use speculoos::prelude::*;

use super::{render, Parameters};
use crate::config::{Config, LinkType};

#[rstest]
#[case("{{ config.variables.test }}", "test")]
#[case("{{ config.variables.nested.nest }}", "nest")]
#[case("{{ whoami.username }}", &whoami::username())]
#[case("{{ dirs.user.home }}", &directories::UserDirs::new().unwrap().home_dir().to_string_lossy().to_string())]
#[case("{{ os }}", &crate::helpers::os::OS.to_string().to_ascii_lowercase())]
fn templating(#[case] template: &str, #[case] expected: &str) {
  assert_that!(render(
    template,
    &Parameters {
      config: &Config {
        dotfiles: "dotfiles".into(),
        link_type: LinkType::Hard,
        shell_command: "shell_command".to_owned().into(),
        variables: map! {
          "test".to_owned() => "test".into(),
          "nested".to_owned() => map!{
            "nest" => value::Value::from("nest")
          }.into()
        }
      },
      name: "name"
    }
  )
  .unwrap())
  .is_equal_to(expected.to_owned());
}
