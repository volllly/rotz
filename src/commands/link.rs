use std::{fs, path::Path};

use directories::UserDirs;

use crate::{
  config::{Config, LinkType},
  dot::{Dot, Merge},
  FILE_EXTENSION,
};

pub fn execute(Config { dotfiles, link_type, repo: _ }: Config, force: bool, dots: Vec<String>) {
  let dotfiles = dotfiles;

  let global = match fs::read_to_string(dotfiles.join(format!("dots.{FILE_EXTENSION}"))) {
    Ok(text) => Some(text.parse::<Dot>().unwrap()),
    Err(err) => match err.kind() {
      std::io::ErrorKind::NotFound => None,
      _ => panic!("{}", err),
    },
  };
  let wildcard = dots.contains(&"*".to_string());
  let links = fs::read_dir(&dotfiles)
    .unwrap()
    .map(|d| d.unwrap().path())
    .filter(|p| p.is_dir())
    .filter(|p| wildcard || dots.contains(&p.file_name().unwrap().to_str().unwrap().to_string()))
    .map(|p| (p.file_name().unwrap().to_str().unwrap().to_string(), fs::read_to_string(p.join(format!("dot.{FILE_EXTENSION}")))))
    .filter_map(|f| match f.1 {
      Ok(text) => Some((f.0, text.parse::<Dot>().unwrap())),
      Err(err) => match err.kind() {
        std::io::ErrorKind::NotFound => None,
        _ => panic!("{}", err),
      },
    })
    .map(|d| (d.0, if let Some(global) = &global { global.clone().merge(&d.1) } else { d.1 }))
    .filter_map(|d| d.1.links.map(|l| (d.0, l)));

  for (name, link) in links {
    println!("Linking {name}");

    let base_path = dotfiles.join(name);
    for (from, tos) in link {
      for mut to in tos {
        println!("  {} -> {}", from.display(), to.display());
        let from = base_path.join(&from);
        if to.starts_with("~/") {
          let mut iter = to.iter();
          iter.next();
          to = UserDirs::new().unwrap().home_dir().iter().chain(iter).collect()
        }

        create_link(from, to, &link_type, force);
      }
    }
    println!();
  }
}

fn create_link<T: AsRef<Path>>(from: T, to: T, link_type: &LinkType, force: bool) {
  let create: fn(&T, &T) -> std::result::Result<(), std::io::Error> = if link_type.is_symbolic() { symlink } else { hardlink };

  match create(&from, &to) {
    Ok(ok) => Ok(ok),
    Err(err) => {
      if force {
        match err.kind() {
          std::io::ErrorKind::AlreadyExists => {
            if to.as_ref().is_dir() {
            fs::remove_dir_all(&to).unwrap();
            } else {
            fs::remove_file(&to).unwrap();
            }
            create(&from, &to)
          }
          _ => Err(err),
        }
      } else { Err(err) }
    }
  }
  .unwrap()
}

#[cfg(windows)]
fn symlink<T: AsRef<Path>>(from: &T, to: &T) -> std::io::Result<()> {
  use std::os::windows::fs;
  if from.as_ref().is_dir() {
    fs::symlink_dir(from, to)?
  } else {
    fs::symlink_file(from, to)?
  };
  Ok(())
}

#[cfg(unix)]
fn symlink<T: AsRef<Path>>(from: &T, to: &T) -> std::io::Result<()> {
  use std::os::unix::fs;
  fs::symlink(from, to)?;
  Ok(())
}

fn hardlink<T: AsRef<Path>>(from: &T, to: &T) -> std::io::Result<()> {
  if from.as_ref().is_dir() {
    junction::create(from, to)?;
  } else {
    fs::hard_link(from, to)?;
  }
  Ok(())
}
