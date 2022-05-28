use std::collections::{HashMap, HashSet};

use indexmap::{indexset, IndexSet};
use miette::{Diagnostic, Result};
use somok::Somok;

use crate::{config::Config, dot::Installs};

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("{name} has a cyclic dependency")]
  #[diagnostic(code(dependency::cyclic), help("{name} depends on itsself through {through}"))]
  CyclicDependency { name: String, through: String },

  #[error("Dependency {0} of {1} was not found")]
  #[diagnostic(code(dependency::not_found))]
  DependencyNotFound(String, String),
}

pub struct Install {
  config: Config,
}

impl Install {
  pub const fn new(config: crate::config::Config) -> Self {
    Self { config }
  }
}

type InstallsDots = (Option<Installs>, Option<HashSet<String>>);

impl super::Command for Install {
  type Args = crate::cli::Install;
  type Result = Result<()>;

  fn execute(&self, link_command: Self::Args) -> Self::Result {
    let dots = crate::dot::read_dots(&self.config.dotfiles, &link_command.dots)?
      .into_iter()
      .filter(|d| d.1.installs.is_some() || d.1.depends.is_some())
      .map(|d| (d.0, (d.1.installs, d.1.depends)))
      .collect::<HashMap<String, InstallsDots>>();

    install_dependencies(&dots)?;

    todo!();

    ().okay()
  }
}

fn install_dependencies(dots: &HashMap<String, InstallsDots>) -> Result<()> {
  detect_cyclic_dependencies(dots)?;

  let installed: HashSet<&str> = HashSet::new();

  // for (name, install, depends) in dots {

  //   install_dependencies(name, install)?;
  // }

  ().okay()
}

fn detect_cyclic_dependencies(dots: &HashMap<String, InstallsDots>) -> Result<()> {
  fn recurse_dependencies<'a>(dots: &HashMap<String, InstallsDots>, dot: (&'a String, &InstallsDots), stack: &'a mut IndexSet<&'a String>) -> Result<()> {
    if stack.contains(dot.0) {
      Error::CyclicDependency {
        name: stack.first().unwrap().to_string(),
        through: stack.last().unwrap().to_string(),
      }
      .error()?;
    }

    stack.insert(dot.0);

    if let Some(installs) = &dot.1 .0 {
      for dependency in installs.depends.iter() {
        recurse_dependencies(
          dots,
          (
            dependency,
            dots.get(dependency.as_str()).ok_or_else(|| Error::DependencyNotFound(dependency.to_string(), dot.0.clone()))?,
          ),
          &mut stack.clone(),
        )?
      }
    }

    if let Some(depends) = &dot.1 .1 {
      for dependency in depends.iter() {
        recurse_dependencies(
          dots,
          (
            dependency,
            dots.get(dependency.as_str()).ok_or_else(|| Error::DependencyNotFound(dependency.to_string(), dot.0.clone()))?,
          ),
          &mut stack.clone(),
        )?
      }
    }

    ().okay()
  }

  for dot in dots.iter() {
    recurse_dependencies(dots, dot, &mut indexset![])?
  }

  ().okay()
}
