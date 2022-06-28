use std::collections::{HashMap, HashSet};

use itertools::Itertools;
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
  let installed = detect_cyclic_dependencies(dots)?;

  println!("{:?}", installed.iter().unique().collect_vec());
  // for (name, install, depends) in dots {

  //   install_dependencies(name, install)?;
  // }

  ().okay()
}

fn detect_cyclic_dependencies(dots: &HashMap<String, InstallsDots>) -> Result<Vec<&str>> {
  fn recurse_dependencies<'a>(dots: &'a HashMap<String, InstallsDots>, dot: (&'a String, &'a InstallsDots), mut stack: Vec<&'a str>) -> Result<Vec<&'a str>> {
    if stack.contains(&dot.0.as_str()) {
      Error::CyclicDependency {
        name: stack.first().unwrap().to_string(),
        through: stack.last().unwrap().to_string(),
      }
      .error()?;
    }

    stack.push(dot.0);

    let old_stack = stack.clone();

    if let Some(installs) = &dot.1 .0 {
      for dependency in installs.depends.iter() {
        let mut tmp = recurse_dependencies(
          dots,
          (
            dependency,
            dots.get(dependency.as_str()).ok_or_else(|| Error::DependencyNotFound(dependency.to_string(), dot.0.clone()))?,
          ),
          old_stack.clone(),
        )?;
        tmp.extend(stack);
        stack = tmp;
      }
    }

    if let Some(depends) = &dot.1 .1 {
      for dependency in depends.iter() {
        let mut tmp = recurse_dependencies(
          dots,
          (
            dependency,
            dots.get(dependency.as_str()).ok_or_else(|| Error::DependencyNotFound(dependency.to_string(), dot.0.clone()))?,
          ),
          old_stack.clone(),
        )?;
        tmp.extend(stack);
        stack = tmp;
      }
    }

    stack.okay()
  }

  let mut stack = vec![];

  for dot in dots.iter() {
    stack.extend(recurse_dependencies(dots, dot, vec![])?);
  }

  stack.okay()
}
