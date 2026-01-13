use speculoos::prelude::*;
use std::sync::LazyLock;

use crate::{
  cli::{Cli, Command},
  config::Config,
  templating::{Engine, Parameters},
};

static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::default());

fn get_parameters<'a>() -> Parameters<'a> {
  Parameters { config: &CONFIG, name: "name" }
}

fn get_handlebars<'a>() -> Engine<'a> {
  let cli = Cli {
    dry_run: true,
    dotfiles: None,
    config: crate::cli::PathBuf("".into()),
    command: Command::Clone { repo: String::new() },
  };

  Engine::new(&Config::default(), &cli)
}

#[test]
fn empty_dot_file() {
  let dot = crate::parse!("yaml", &get_handlebars(), &get_parameters());

  // Verify empty file produces default behavior
  assert_that!(dot.installs).is_none();
  assert_that!(dot.links).is_none();
  assert_that!(dot.depends).is_none();
}

#[test]
fn empty_dot_file_equivalent_to_global_false() {
  let dot = crate::parse!("yaml", &get_handlebars(), &get_parameters());

  // This should be equivalent to:
  // global:
  //   installs: false
  //   depends: []
  //   links: {}

  // Default behavior should be no operations
  assert_that!(dot.installs).is_none(); // No installs
  assert_that!(dot.depends).is_none(); // No dependencies
  assert_that!(dot.links).is_none(); // No links
}

#[test]
fn empty_file_comprehensive_default_behavior() {
  let dot = crate::parse!("yaml", &get_handlebars(), &get_parameters());

  // An empty dot.yaml should be equivalent to:
  // global:
  //   installs: false
  //   depends: []
  //   links: {}

  // Verify all fields are None (no operations)
  assert_that!(dot.installs).is_none();
  assert_that!(dot.depends).is_none();
  assert_that!(dot.links).is_none();

  // This means:
  // - No packages will be installed
  // - No dependencies will be processed
  // - No symlinks will be created
  // - The dot directory is effectively a no-op
}
