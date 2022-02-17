use std::{
  fs,
  path::{Path, PathBuf},
};

use color_eyre::eyre::{Result, WrapErr};
use crossterm::style::{Attribute, Stylize};
use directories::UserDirs;
use itertools::Itertools;
use somok::Somok;

use crate::{
  config::{Config, LinkType},
  dot::{Dot, Merge},
  FILE_EXTENSION, USER_DIRS,
};

#[derive(thiserror::Error, Debug)]
enum Error {
  #[error("{0}")]
  Path(String),
  #[cfg(feature = "yaml")]
  #[error(transparent)]
  Parse(#[from] serde_yaml::Error),
  #[error("Io Error on file {0}")]
  Io(PathBuf, #[source] std::io::Error),
  #[error("Could not create link from {0} to {1}")]
  Symlink(PathBuf, PathBuf, #[source] std::io::Error),
}

pub fn execute(Config { dotfiles, link_type, repo: _ }: Config, force: bool, dots: Vec<String>) -> Result<()> {
  let dotfiles = dotfiles;

  let global = match fs::read_to_string(dotfiles.join(format!("dots.{FILE_EXTENSION}"))) {
    Ok(text) => text.parse::<Dot>()?.some(),
    Err(err) => match err.kind() {
      std::io::ErrorKind::NotFound => None,
      _ => panic!("{}", err),
    },
  };
  let wildcard = dots.contains(&"*".to_string());
  let paths = fs::read_dir(&dotfiles)
    .wrap_err(format!("Could not read dotfiles directory {}", dotfiles.display()))?
    .map_ok(|d| d.path())
    .filter_ok(|p| p.is_dir());

  let dotsfile = crate::helpers::join_err_result(paths.collect())?
    .into_iter()
    .map(|p| {
      let name = p
        .file_name()
        .ok_or_else(|| Error::Path("Could not find dot directory".to_string()))?
        .to_str()
        .ok_or_else(|| Error::Path("Could not parse dot directory".to_string()))?
        .to_string();
      Ok::<(String, PathBuf), Error>((name, p))
    })
    .filter_ok(|p| wildcard || dots.contains(&p.0))
    .map_ok(|p| {
      (
        p.0,
        fs::read_to_string(p.1.join(format!("dot.{FILE_EXTENSION}"))).map_err(|e| Error::Io(p.1.join(format!("dot.{FILE_EXTENSION}")), e)),
      )
    });

  let dots = dotsfile.filter_map(|f| match f {
    Ok((name, Ok(text))) => match text.parse::<Dot>() {
      Ok(dot) => (name, dot).okay().some(),
      Err(err) => Error::Parse(err).error().some(),
    },
    Ok((_, Err(Error::Io(file, err)))) => match err.kind() {
      std::io::ErrorKind::NotFound => None,
      _ => Error::Io(file, err).error().some(),
    },
    Ok((_, Err(err))) => err.error().some(),
    Err(err) => err.error().some(),
  });

  let links = dots
    .map_ok(|d| (d.0, if let Some(global) = &global { global.clone().merge(&d.1) } else { d.1 }))
    .filter_map_ok(|d| d.1.links.map(|l| (d.0, l)));

  let links = crate::helpers::join_err_result(links.collect())?;
  if links.is_empty() {
    println!("Warning: {}", "No dots found".yellow());
    return ().okay();
  }

  let mut errors = Vec::<Error>::new();
  for (name, link) in links.into_iter() {
    println!("{}Linking {}{}\n", Attribute::Bold, name.as_str().blue(), Attribute::Reset);

    let base_path = dotfiles.join(name);
    for (from, tos) in link {
      for mut to in tos {
        println!("  {} -> {}", from.display().to_string().green(), to.display().to_string().green());
        let from = base_path.join(&from);
        if to.starts_with("~/") {
          let mut iter = to.iter();
          iter.next();
          to = USER_DIRS.home_dir().iter().chain(iter).collect()
        }

        if let Err(err) = create_link(from, to, &link_type, force) {
          errors.push(err)
        }
      }
    }
    println!();
  }

  crate::helpers::join_err(errors)
}

fn create_link<T: AsRef<Path>>(from: T, to: T, link_type: &LinkType, force: bool) -> std::result::Result<(), Error> {
  let create: fn(&T, &T) -> std::result::Result<(), std::io::Error> = if link_type.is_symbolic() { symlink } else { hardlink };

  match create(&from, &to) {
    Ok(ok) => ok.okay(),
    Err(err) => {
      if force {
        match err.kind() {
          std::io::ErrorKind::AlreadyExists => {
            if to.as_ref().is_dir() { fs::remove_dir_all(&to) } else { fs::remove_file(&to) }.map_err(|e| Error::Symlink(from.as_ref().to_path_buf(), to.as_ref().to_path_buf(), e))?;
            create(&from, &to)
          }
          _ => err.error(),
        }
      } else {
        err.error()
      }
    }
  }
  .map_err(|e| Error::Symlink(from.as_ref().to_path_buf(), to.as_ref().to_path_buf(), e))
}

#[cfg(windows)]
fn symlink<T: AsRef<Path>>(from: &T, to: &T) -> std::io::Result<()> {
  use std::os::windows::fs;
  if from.as_ref().is_dir() {
    fs::symlink_dir(from, to)?
  } else {
    fs::symlink_file(from, to)?
  };
  ().okay()
}

#[cfg(unix)]
fn symlink<T: AsRef<Path>>(from: &T, to: &T) -> std::io::Result<()> {
  use std::os::unix::fs;
  fs::symlink(from, to)?;
  ().okay()
}

#[cfg(windows)]
fn hardlink<T: AsRef<Path>>(from: &T, to: &T) -> std::io::Result<()> {
  if from.as_ref().is_dir() {
    junction::create(from, to)?;
  } else {
    fs::hard_link(from, to)?;
  }
  ().okay()
}

#[cfg(unix)]
fn hardlink<T: AsRef<Path>>(from: &T, to: &T) -> std::io::Result<()> {
  fs::hard_link(from, to)?;
  ().okay()
}
