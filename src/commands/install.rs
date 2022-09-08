use std::{
  collections::{HashMap, HashSet},
  path::{Path, PathBuf},
};

use crossterm::style::{Attribute, Stylize};
use indexmap::IndexSet;
use miette::{Diagnostic, Report, Result};
use path_slash::PathBufExt;
use serde_json::json;
use somok::Somok;
use wax::Pattern;

use super::Command;
use crate::{config::Config, dot::Installs, helpers, templating::HANDLEBARS};

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("{name} has a cyclic dependency")]
  #[diagnostic(code(dependency::cyclic), help("{} depends on itsself through {}", name.to_slash_lossy(), through.to_slash_lossy()))]
  CyclicDependency { name: PathBuf, through: PathBuf },

  #[error("{name} has a cyclic installation dependency")]
  #[diagnostic(code(dependency::cyclic::install), help("{} depends on itsself through {}", name.to_slash_lossy(), through.to_slash_lossy()))]
  CyclicInstallDependency { name: PathBuf, through: PathBuf },

  #[error("Dependency {1} of {0} was not found")]
  #[diagnostic(code(dependency::not_found))]
  DependencyNotFound(PathBuf, PathBuf),

  #[error("Install command for {0} did not run successfully")]
  #[diagnostic(code(install::command::run))]
  InstallExecute(PathBuf, #[source] helpers::RunError),

  #[error("Could not render command templeate for {0}")]
  #[diagnostic(code(install::command::render))]
  RenderingTemplate(PathBuf, #[source] handlebars::RenderError),

  #[error("Could not parse install command for {0}")]
  #[diagnostic(code(install::command::parse))]
  ParsingInstallCommand(PathBuf, #[source] shellwords::MismatchedQuotes),
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
    dots: &'a HashMap<PathBuf, InstallsDots>,
    entry: (&'a PathBuf, &'a InstallsDots),
    installed: &mut HashSet<&'a Path>,
    mut stack: IndexSet<&'a Path>,
    (globals, install_command): (&crate::cli::Globals, &crate::cli::Install),
  ) -> Result<(), Error> {
    if installed.contains(entry.0.as_path()) {
      return ().okay();
    }

    stack.insert(entry.0.as_path());

    if let Some(installs) = &entry.1 .0 {
      if !(install_command.skip_all_dependencies || install_command.skip_installation_dependencies) {
        for dependency in &installs.depends {
          if stack.contains(dependency.as_path()) {
            return Error::CyclicInstallDependency {
              name: dependency.clone(),
              through: entry.0.clone(),
            }
            .error();
          }

          self.install(
            dots,
            (
              dependency,
              dots.get(dependency.as_path()).ok_or_else(|| Error::DependencyNotFound(entry.0.clone(), dependency.clone()))?,
            ),
            installed,
            stack.clone(),
            (globals, install_command),
          )?;
        }
      }

      println!("{}Installing {}{}\n", Attribute::Bold, entry.0.to_string_lossy().blue(), Attribute::Reset);

      let inner_cmd = installs.cmd.clone();

      let cmd = if let Some(shell_command) = self.config.shell_command.as_ref() {
        HANDLEBARS
          .render_template(shell_command, &json!({ "cmd": &inner_cmd }))
          .map_err(|err| Error::RenderingTemplate(entry.0.clone(), err))?
      } else {
        inner_cmd.clone()
      };

      let cmd = shellwords::split(&cmd).map_err(|err| Error::ParsingInstallCommand(entry.0.clone(), err))?;

      println!("{}{}{}\n", Attribute::Italic, inner_cmd, Attribute::Reset);

      if let Err(err) = helpers::run_command(&cmd[0], &cmd[1..], false, globals.dry_run) {
        if let helpers::RunError::Spawn(err) = &err {
          if err.kind() == std::io::ErrorKind::NotFound {
            println!("kek");
          }
        }

        let error = Error::InstallExecute(entry.0.clone(), err);

        if install_command.continue_on_error {
          eprintln!("\n Error: {:?}", Report::new(error));
        } else {
          return error.error();
        }
      }

      installed.insert(entry.0.as_path());
    }

    if !(install_command.skip_all_dependencies || install_command.skip_dependencies) {
      if let Some(dependencies) = &entry.1 .1 {
        for dependency in dependencies {
          if stack.contains(dependency.as_path()) {
            return Error::CyclicDependency {
              name: dependency.clone(),
              through: entry.0.clone(),
            }
            .error();
          }

          self.install(
            dots,
            (
              dependency,
              dots.get(dependency.as_path()).ok_or_else(|| Error::DependencyNotFound(entry.0.clone(), dependency.clone()))?,
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

type InstallsDots = (Option<Installs>, Option<HashSet<PathBuf>>);

impl Command for Install {
  type Args = (crate::cli::Globals, crate::cli::Install);
  type Result = Result<()>;

  fn execute(&self, (globals, install_command): Self::Args) -> Self::Result {
    let dots = crate::dot::read_dots(&self.config.dotfiles, &["**".to_owned()], &self.config)?
      .into_iter()
      .filter(|d| d.1.installs.is_some() || d.1.depends.is_some())
      .map(|d| (d.0, (d.1.installs, d.1.depends)))
      .collect::<HashMap<PathBuf, InstallsDots>>();

    let mut installed: HashSet<&Path> = HashSet::new();
    let globs = helpers::glob_from_vec(&install_command.dots, "/dot.{ya?ml,toml,json}")?;
    for dot in &dots {
      if globs.is_match(dot.0.as_path()) {
        self.install(&dots, dot, &mut installed, IndexSet::new(), (&globals, &install_command))?;
      }
    }

    ().okay()
  }
}
