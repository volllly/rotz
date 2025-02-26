use crate::{
  cli::{Cli, Command},
  config::{Config, LinkType},
  templating::{Engine, Parameters},
};
use figment::{util::map, value};
use once_cell::sync::Lazy;

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
    config: crate::cli::PathBuf("".into()),
    command: Command::Clone { repo: String::new() },
  };

  Engine::new(&Config::default(), &cli)
}

mod f01;
mod f02;
mod f03;
mod f04;
mod f05;
