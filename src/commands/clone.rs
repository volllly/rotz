use std::process::Command;

use color_eyre::eyre::{eyre, Result};
use crossterm::style::{Stylize};
use somok::Somok;

use crate::{
  config::Config,
};

#[derive(thiserror::Error, Debug)]
enum Error {}

pub fn execute(Config { dotfiles, link_type: _, repo }: Config) -> Result<()> {
  Command::new("git")
    .args([
      "clone",
      repo.as_ref().ok_or_else(|| eyre!("No repo set"))?,
      dotfiles.as_os_str().to_str().ok_or_else(|| eyre!("Invalid dotfiles path"))?,
    ])
    .output()?;

  println!("Cloned {}\n    to {}", repo.unwrap().blue(), dotfiles.display().to_string().green());
  
  ().okay()
}
