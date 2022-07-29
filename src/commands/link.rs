use std::{
  fs,
  path::{Path, PathBuf},
};

use crossterm::style::{Attribute, Stylize};
use miette::{Diagnostic, Report, Result};
use somok::Somok;

use crate::{
  config::{Config, LinkType},
  USER_DIRS,
};

#[derive(thiserror::Error, Diagnostic, Debug)]
enum Error {
  #[error("Could not create link from \"{0}\" to \"{1}\"")]
  #[cfg_attr(windows, diagnostic(code(link::linking), help("You may need to run Rotz from an admin shell to create file links")))]
  #[cfg_attr(not(windows), diagnostic(code(link::linking),))]
  Symlink(PathBuf, PathBuf, #[source] std::io::Error),

  #[error("The file \"{0}\" already exists")]
  #[diagnostic(code(link::already_exists), help("Try using the --force flag"))]
  AlreadyExists(PathBuf),
}

pub struct Link {
  config: Config,
}

impl Link {
  pub const fn new(config: crate::config::Config) -> Self {
    Self { config }
  }
}

impl super::Command for Link {
  type Args = (crate::cli::Globals, crate::cli::Link);
  type Result = Result<()>;

  fn execute(&self, (globals, link_command): Self::Args) -> Self::Result {
    let links = crate::dot::read_dots(&self.config.dotfiles, &link_command.dots, &self.config)?
      .into_iter()
      .filter_map(|d| d.1.links.map(|l| (d.0, l)));

    for (name, link) in links {
      println!("{}Linking {}{}\n", Attribute::Bold, name.as_str().blue(), Attribute::Reset);

      let base_path = self.config.dotfiles.join(name);
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
            if let Err(err) = create_link(&from, &to, &self.config.link_type, link_command.force) {
              eprintln!("\n Error: {:?}", Report::new(err));
            }
          }
        }
      }
      println!();
    }

    Ok(())
  }
}

fn create_link(from: &Path, to: &Path, link_type: &LinkType, force: bool) -> std::result::Result<(), Error> {
  let create: fn(&Path, &Path) -> std::result::Result<(), std::io::Error> = if link_type.is_symbolic() { symlink } else { hardlink };

  match create(from, to) {
    Ok(ok) => ok.okay(),
    Err(err) => match err.kind() {
      std::io::ErrorKind::AlreadyExists => {
        if force {
          if to.is_dir() { fs::remove_dir_all(&to) } else { fs::remove_file(&to) }.map_err(|e| Error::Symlink(from.to_path_buf(), to.to_path_buf(), e))?;
          create(from, to)
        } else {
          return Error::AlreadyExists(to.to_path_buf()).error();
        }
      }
      _ => err.error(),
    },
  }
  .map_err(|e| Error::Symlink(from.to_path_buf(), to.to_path_buf(), e))
}

#[cfg(windows)]
fn symlink(from: &Path, to: &Path) -> std::io::Result<()> {
  if let Some(parent) = to.parent() {
    std::fs::create_dir_all(parent)?;
  }

  use std::os::windows::fs;
  if from.is_dir() {
    fs::symlink_dir(from, to)?;
  } else {
    fs::symlink_file(from, to)?;
  };
  ().okay()
}

#[cfg(unix)]
fn symlink(from: &Path, to: &Path) -> std::io::Result<()> {
  use std::os::unix::fs;
  if let Some(parent) = to.parent() {
    std::fs::create_dir_all(parent)?;
  }
  fs::symlink(from, to)?;
  ().okay()
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
  ().okay()
}

#[cfg(unix)]
fn hardlink(from: &Path, to: &Path) -> std::io::Result<()> {
  if let Some(parent) = to.parent() {
    std::fs::create_dir_all(parent)?;
  }
  fs::hard_link(from, to)?;
  ().okay()
}
