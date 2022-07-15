use std::collections::{HashMap, HashSet};

use crossterm::style::{Attribute, Stylize};
use handlebars::Handlebars;
use indexmap::IndexSet;
use miette::{Diagnostic, Result};
use once_cell::sync::Lazy;
use serde_json::json;
use somok::Somok;

use super::Command;
use crate::{config::Config, dot::Installs, helpers};

pub(crate) static HANDLEBARS: Lazy<Handlebars> = Lazy::new(handlebars_misc_helpers::new_hbs);

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("{name} has a cyclic dependency")]
  #[diagnostic(code(dependency::cyclic), help("{name} depends on itsself through {through}"))]
  CyclicDependency { name: String, through: String },

  #[error("{name} has a cyclic installation dependency")]
  #[diagnostic(code(dependency::cyclic::install), help("{name} depends on itsself through {through}"))]
  CyclicInstallDependency { name: String, through: String },

  #[error("Dependency {1} of {0} was not found")]
  #[diagnostic(code(dependency::not_found))]
  DependencyNotFound(String, String),

  #[error("Install command for {0} did not run successfully")]
  #[diagnostic(code(install::command::run))]
  InstallExecute(String, #[source] helpers::RunError),

  #[error("Could not render command templeate for {0}")]
  #[diagnostic(code(install::command::render))]
  RenderingTemplate(String, #[source] handlebars::RenderError),

  #[error("Could not parse install command for {0}")]
  #[diagnostic(code(install::command::parse))]
  ParsingInstallCommand(String, #[source] shellwords::MismatchedQuotes),
}

pub struct Install {
  config: Config,
}

impl Install {
  pub const fn new(config: crate::config::Config) -> Self {
    Self { config }
  }

  fn install<'a>(
    &self,
    dots: &'a HashMap<String, InstallsDots>,
    entry: (&'a String, &'a InstallsDots),
    installed: &mut HashSet<&'a str>,
    mut stack: IndexSet<&'a str>,
    (globals, install_command): (&crate::cli::Globals, &crate::cli::Install),
  ) -> Result<(), Error> {
    if installed.contains(entry.0.as_str()) {
      return ().okay();
    }

    stack.insert(entry.0.as_str());

    if let Some(installs) = &entry.1 .0 {
      if !(install_command.skip_all_dependencies || install_command.skip_installation_dependencies) {
        for dependency in &installs.depends {
          if stack.contains(dependency.as_str()) {
            return Error::CyclicInstallDependency {
              name: dependency.to_string(),
              through: entry.0.to_string(),
            }
            .error();
          }

          self.install(
            dots,
            (
              dependency,
              dots.get(dependency.as_str()).ok_or_else(|| Error::DependencyNotFound(entry.0.to_string(), dependency.to_string()))?,
            ),
            installed,
            stack.clone(),
            (globals, install_command),
          )?;
        }
      }

      println!("{}Installing {}{}\n", Attribute::Bold, entry.0.as_str().blue(), Attribute::Reset);

      let inner_cmd = HANDLEBARS
        .render_template(
          &installs.cmd.to_string(),
          &json!({
            "name": entry.0
          }),
        )
        .map_err(|err| Error::RenderingTemplate(entry.0.to_string(), err))?;

      let cmd = if let Some(shell_command) = self.config.shell_command.as_ref() {
        HANDLEBARS
          .render_template(
            shell_command,
            &json!({
              "name": entry.0,
              "cmd": &inner_cmd
            }),
          )
          .map_err(|err| Error::RenderingTemplate(entry.0.to_string(), err))?
      } else {
        inner_cmd.clone()
      };

      let cmd = shellwords::split(&cmd).map_err(|err| Error::ParsingInstallCommand(entry.0.to_string(), err))?;

      println!("{}{}{}\n", Attribute::Italic, inner_cmd, Attribute::Reset);

      if let Err(err) = helpers::run_command(&cmd[0], &cmd[1..], false, globals.dry_run) {
        if !install_command.continue_on_error {
          return Error::InstallExecute(entry.0.to_string(), err).error();
        }
      }

      installed.insert(entry.0.as_str());
    }

    if !(install_command.skip_all_dependencies || install_command.skip_dependencies) {
      if let Some(dependencies) = &entry.1 .1 {
        for dependency in dependencies {
          if stack.contains(dependency.as_str()) {
            return Error::CyclicDependency {
              name: dependency.to_string(),
              through: entry.0.to_string(),
            }
            .error();
          }

          self.install(
            dots,
            (
              dependency,
              dots.get(dependency.as_str()).ok_or_else(|| Error::DependencyNotFound(entry.0.to_string(), dependency.to_string()))?,
            ),
            installed,
            stack.clone(),
            (globals, install_command),
          )?;
        }
      }
    }

    ().okay()
  }
}

type InstallsDots = (Option<Installs>, Option<HashSet<String>>);

impl Command for Install {
  type Args = (crate::cli::Globals, crate::cli::Install);
  type Result = Result<()>;

  fn execute(&self, (globals, install_command): Self::Args) -> Self::Result {
    let dots = crate::dot::read_dots(&self.config.dotfiles, &["*".to_string()])?
      .into_iter()
      .filter(|d| d.1.installs.is_some() || d.1.depends.is_some())
      .map(|d| (d.0, (d.1.installs, d.1.depends)))
      .collect::<HashMap<String, InstallsDots>>();

    let mut installed: HashSet<&str> = HashSet::new();
    for dot in dots.iter() {
      if install_command.dots.contains(&"*".to_string()) || install_command.dots.contains(dot.0) {
        self.install(&dots, dot, &mut installed, IndexSet::new(), (&globals, &install_command))?;
      }
    }

    ().okay()
  }
}
