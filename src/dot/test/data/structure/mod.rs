use crate::{
  cli::{Cli, Command, PathBuf},
  config::{Config, LinkType},
  templating::{Engine, Parameters},
};
use figment::{util::map, value};
use once_cell::sync::Lazy;

#[macro_export]
macro_rules! parse {
  ($format:literal, $engine:expr, $parameters:expr) => {
    $crate::dot::from_str_with_defaults(
      std::fs::read_to_string(std::path::Path::new(file!()).parent().unwrap().join(format!("dot.{}", $format)))
        .unwrap()
        .as_str(),
      $crate::FileFormat::try_from($format).unwrap(),
      None,
      $engine,
      $parameters,
    )
    .unwrap()
  };
}

static CONFIG: Lazy<Config> = Lazy::new(|| Config {
  dotfiles: "dotfiles".into(),
  link_type: LinkType::Hard,
  shell_command: "shell_command".to_owned().into(),
  variables: map! {
    "test".to_owned() => "test".into(),
    "nested".to_owned() => map!{
      "nest" => value::Value::from("nest")
    }.into()
  },
});

pub(crate) fn get_parameters<'a>() -> Parameters<'a> {
  Parameters { config: &CONFIG, name: "name" }
}

pub(crate) fn get_handlebars<'a>() -> Engine<'a> {
  let cli = Cli {
    dry_run: true,
    dotfiles: None,
    config: PathBuf("".into()),
    command: Command::Clone { repo: String::new() },
  };

  Engine::new(&Config::default(), &cli)
}

mod s01;
mod s02;
mod s03;
mod s04;
mod s05;
mod s06;
mod s07;
