use std::{
  collections::{HashMap, HashSet},
  fs,
  path::{Path, PathBuf},
};

use crossterm::style::{Attribute, Stylize};
use itertools::Itertools;
use miette::{Diagnostic, Report, Result};
use tap::Pipe;
use velcro::hash_map;

use super::Command;
use crate::{
  config::{Config, LinkType},
  helpers,
  state::State,
  templating, USER_DIRS,
};

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("Could not create link from \"{0}\" to \"{1}\"")]
  #[cfg_attr(windows, diagnostic(code(link::linking), help("You may need to run Rotz from an admin shell to create file links")))]
  #[cfg_attr(not(windows), diagnostic(code(link::linking),))]
  Symlink(PathBuf, PathBuf, #[source] std::io::Error),

  #[error("Could not remove orphaned link from \"{0}\" to \"{1}\"")]
  #[diagnostic(code(link::orphan::remove))]
  RemovingOrphan(PathBuf, PathBuf, #[source] std::io::Error),

  #[error("The file \"{0}\" already exists")]
  #[diagnostic(code(link::already_exists), help("Try using the --force flag"))]
  AlreadyExists(PathBuf),
}

pub(crate) struct Link<'a> {
  config: Config,
  engine: templating::Engine<'a>,
}

impl<'a> Link<'a> {
  pub const fn new(config: crate::config::Config, engine: templating::Engine<'a>) -> Self {
    Self { config, engine }
  }
}

impl<'a> Command for Link<'a> {
  type Args = (crate::cli::Globals, crate::cli::Link, State);
  type Result = Result<State>;

  fn execute(&self, (globals, link_command, State { linked }): Self::Args) -> Self::Result {
    let links = crate::dot::read_dots(&self.config.dotfiles, &link_command.dots, &self.config, &self.engine)?
      .into_iter()
      .filter_map(|d| d.1.links.map(|l| (d.0, l)))
      .collect_vec();

    {
      let current_links = links
        .iter()
        .flat_map(|l| l.1.iter().map(|h| h.1.iter()))
        .flatten()
        .map(|l| {
          if l.starts_with("~/") {
            let mut iter = l.iter();
            iter.next();
            USER_DIRS.home_dir().iter().chain(iter).collect()
          } else {
            l.clone()
          }
        })
        .collect::<HashSet<_>>();

      let mut errors = Vec::new();

      for (name, links) in &linked {
        let mut printed = false;
        for (to, from) in links {
          if !current_links.contains(to) {
            if !printed {
              println!("{}Removing orphans for {}{}\n", Attribute::Bold, name.as_str().blue(), Attribute::Reset);
              printed = true;
            }
            println!("  x {}", to.to_string_lossy().green());

            if !globals.dry_run {
              fs::remove_file(&to)
                .map_err(|err| Error::RemovingOrphan(from.clone(), to.clone(), err))
                .map_err(|err| errors.push(err))
                .ok();
            }
          }
        }

        if printed {
          println!();
        }
      }

      helpers::join_err(errors)?;
    }

    let mut new_linked = hash_map!();

    for (name, link) in links {
      println!("{}Linking {}{}\n", Attribute::Bold, name.as_str().blue(), Attribute::Reset);

      let mut new_linked_inner = hash_map!();

      let base_path = self.config.dotfiles.join(&name[1..]);
      for (from, tos) in link {
        for mut to in tos {
          println!("  {} -> {}", from.to_string_lossy().green(), to.to_string_lossy().green());
          let from = base_path.join(&from);
          if to.starts_with("~/") {
            let mut iter = to.iter();
            iter.next();
            to = USER_DIRS.home_dir().iter().chain(iter).collect();
          }

          if !globals.dry_run {
            if let Err(err) = create_link(&from, &to, &self.config.link_type, link_command.force, linked.get(&name)) {
              eprintln!("\n Error: {:?}", Report::new(err));
            } else {
              new_linked_inner.insert(to.clone(), from.clone());
            }
          }
        }
      }

      if !new_linked_inner.is_empty() {
        new_linked.insert(name, new_linked_inner);
      }

      println!();
    }

    Ok(State { linked: new_linked })
  }
}

fn create_link(from: &Path, to: &Path, link_type: &LinkType, force: bool, linked: Option<&HashMap<PathBuf, PathBuf>>) -> std::result::Result<(), Error> {
  let create: fn(&Path, &Path) -> std::result::Result<(), std::io::Error> = if link_type.is_symbolic() { symlink } else { hardlink };

  match create(from, to) {
    Ok(ok) => ok.pipe(Ok),
    Err(err) => match err.kind() {
      std::io::ErrorKind::AlreadyExists => {
        if force || linked.map_or(false, |l| l.contains_key(to)) {
          if to.is_dir() { fs::remove_dir_all(&to) } else { fs::remove_file(&to) }.map_err(|e| Error::Symlink(from.to_path_buf(), to.to_path_buf(), e))?;
          create(from, to)
        } else {
          return Error::AlreadyExists(to.to_path_buf()).pipe(Err);
        }
      }
      _ => err.pipe(Err),
    },
  }
  .map_err(|e| Error::Symlink(from.to_path_buf(), to.to_path_buf(), e))
}

#[cfg(windows)]
fn symlink(from: &Path, to: &Path) -> std::io::Result<()> {
  use std::os::windows::fs;

  if let Some(parent) = to.parent() {
    std::fs::create_dir_all(parent)?;
  }

  if from.is_dir() {
    fs::symlink_dir(from, to)?;
  } else {
    fs::symlink_file(from, to)?;
  };
  ().pipe(Ok)
}

#[cfg(unix)]
fn symlink(from: &Path, to: &Path) -> std::io::Result<()> {
  use std::os::unix::fs;
  if let Some(parent) = to.parent() {
    std::fs::create_dir_all(parent)?;
  }
  fs::symlink(from, to)?;
  ().pipe(Ok)
}

#[cfg(windows)]
fn hardlink(from: &Path, to: &Path) -> std::io::Result<()> {
  if let Some(parent) = to.parent() {
    std::fs::create_dir_all(parent)?;
  }

  if from.is_dir() {
    junction::create(from, to)?;
  } else {
    fs::hard_link(from, to)?;
  }
  ().pipe(Ok)
}

#[cfg(unix)]
fn hardlink(from: &Path, to: &Path) -> std::io::Result<()> {
  if let Some(parent) = to.parent() {
    std::fs::create_dir_all(parent)?;
  }
  fs::hard_link(from, to)?;
  ().pipe(Ok)
}
