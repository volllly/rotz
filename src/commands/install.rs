use std::collections::{HashMap, HashSet};

use crossterm::style::{Attribute, Stylize};
use indexmap::IndexSet;
use miette::{Diagnostic, Report, Result};
use somok::Somok;
use velcro::hash_map;
use wax::{Glob, Pattern};

use super::Command;
use crate::{config::Config, dot::Installs, helpers, templating::HANDLEBARS};

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("{name} has a cyclic dependency")]
  #[diagnostic(code(dependency::cyclic), help("{} depends on itsself through {}", name, through))]
  CyclicDependency { name: String, through: String },

  #[error("{name} has a cyclic installation dependency")]
  #[diagnostic(code(dependency::cyclic::install), help("{} depends on itsself through {}", name, through))]
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

  #[error("Could not spawl install command")]
  #[diagnostic(code(install::command::spawn), help("The shell_command in your config is set to \"{0}\" is that correct?"))]
  CouldNotSpawn(String),

  #[error("Could not parse dependency \"{0}\"")]
  #[diagnostic(code(glob::parse))]
  ParseGlob(String, #[source] wax::BuildError<'static>),
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
    mut stack: IndexSet<String>,
    (globals, install_command): (&crate::cli::Globals, &crate::cli::Install),
  ) -> Result<(), Error> {
    if installed.contains(entry.0.as_str()) {
      return ().okay();
    }

    stack.insert(entry.0.clone());

    macro_rules! recurse {
      ($depends:expr, $error:ident) => {
        for dependency in $depends {
          let dependency_glob = Glob::new(dependency).map_err(|e| Error::ParseGlob(dependency.clone(), e.into_owned()))?;

          if stack.iter().any(|d| dependency_glob.is_match(&**d)) {
            return Error::$error {
              name: dependency.clone(),
              through: entry.0.clone(),
            }
            .error();
          }

          self.install(
            dots,
            (
              dependency,
              dots
                .iter()
                .find(|d| dependency_glob.is_match(&**d.0))
                .map(|d| d.1)
                .ok_or_else(|| Error::DependencyNotFound(entry.0.clone(), dependency.clone()))?,
            ),
            installed,
            stack.clone(),
            (globals, install_command),
          )?;
        }
      };
    }

    if let Some(installs) = &entry.1 .0 {
      if !(install_command.skip_all_dependencies || install_command.skip_installation_dependencies) {
        recurse!(&installs.depends, CyclicInstallDependency);
      }

      println!("{}Installing {}{}\n", Attribute::Bold, entry.0.as_str().blue(), Attribute::Reset);

      let inner_cmd = installs.cmd.clone();

      let cmd = if let Some(shell_command) = self.config.shell_command.as_ref() {
        HANDLEBARS
          .render_template(shell_command, &hash_map! { "cmd": &inner_cmd })
          .map_err(|err| Error::RenderingTemplate(entry.0.clone(), err))?
      } else {
        inner_cmd.clone()
      };

      let cmd = shellwords::split(&cmd).map_err(|err| Error::ParsingInstallCommand(entry.0.clone(), err))?;

      println!("{}{}{}\n", Attribute::Italic, inner_cmd, Attribute::Reset);

      if let Err(err) = helpers::run_command(&cmd[0], &cmd[1..], false, globals.dry_run) {
        if let helpers::RunError::Spawn(err) = &err {
          if err.kind() == std::io::ErrorKind::NotFound {
            eprintln!("\n Error: {:?}", Report::new(Error::CouldNotSpawn(format!("{:?}", self.config.shell_command))));
          }
        }

        let error = Error::InstallExecute(entry.0.clone(), err);

        if install_command.continue_on_error {
          eprintln!("\n Error: {:?}", Report::new(error));
        } else {
          return error.error();
        }
      }

      installed.insert(entry.0.as_str());
    }

    if !(install_command.skip_all_dependencies || install_command.skip_dependencies) {
      if let Some(depends) = &entry.1 .1 {
        recurse!(depends, CyclicDependency);
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
    let dots = crate::dot::read_dots(&self.config.dotfiles, &["/**".to_owned()], &self.config)?
      .into_iter()
      .filter(|d| d.1.installs.is_some() || d.1.depends.is_some())
      .map(|d| (d.0, (d.1.installs, d.1.depends)))
      .collect::<HashMap<String, InstallsDots>>();

    let mut installed: HashSet<&str> = HashSet::new();
    let globs = helpers::glob_from_vec(&install_command.dots, "")?;
    for dot in &dots {
      if globs.is_match(dot.0.as_str()) {
        self.install(&dots, dot, &mut installed, IndexSet::new(), (&globals, &install_command))?;
      }
    }

    ().okay()
  }
}
