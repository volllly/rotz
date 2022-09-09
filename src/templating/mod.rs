use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

use directories::BaseDirs;
use handlebars::Handlebars;
use itertools::Itertools;
use miette::Diagnostic;
use once_cell::sync::Lazy;
use serde::Serialize;

use crate::{config::Config, helpers, USER_DIRS};

pub static HANDLEBARS: Lazy<Handlebars> = Lazy::new(|| {
  let mut hb = handlebars_misc_helpers::new_hbs();
  hb.set_strict_mode(false);
  hb
});

pub static ENV: Lazy<HashMap<String, String>> = Lazy::new(|| std::env::vars().collect());

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum Error {
  #[error("Could not render templeate")]
  #[diagnostic(code(template::render))]
  RenderingTemplate(#[source] handlebars::RenderError),
}

#[derive(Serialize)]
pub struct Parameters<'a> {
  pub config: &'a Config,
  pub name: &'a Path,
}

#[derive(Serialize)]
pub struct WhoamiPrameters {
  pub realname: String,
  pub username: String,
  pub lang: Vec<String>,
  pub devicename: String,
  pub hostname: String,
  pub platform: String,
  pub distro: String,
  pub desktop_env: String,
}

pub static WHOAMI_PRAMETERS: Lazy<WhoamiPrameters> = Lazy::new(|| WhoamiPrameters {
  realname: whoami::realname(),
  username: whoami::username(),
  lang: whoami::lang().collect_vec(),
  devicename: whoami::devicename(),
  hostname: whoami::hostname(),
  platform: whoami::platform().to_string(),
  distro: whoami::distro(),
  desktop_env: whoami::desktop_env().to_string(),
});

#[derive(Serialize)]
pub struct DirectoryPrameters {
  pub base: HashMap<&'static str, PathBuf>,
  pub user: HashMap<&'static str, PathBuf>,
}

pub static DIRECTORY_PRAMETERS: Lazy<DirectoryPrameters> = Lazy::new(|| {
  let mut base: HashMap<&'static str, PathBuf> = HashMap::new();

  if let Some(dirs) = BaseDirs::new() {
    base.insert("cache", dirs.cache_dir().to_path_buf());
    base.insert("config", dirs.config_dir().to_path_buf());
    base.insert("data", dirs.data_dir().to_path_buf());
    base.insert("data_local", dirs.data_local_dir().to_path_buf());
    base.insert("home", dirs.home_dir().to_path_buf());
    base.insert("preference", dirs.preference_dir().to_path_buf());
    if let Some(dir) = dirs.executable_dir() {
      base.insert("executable", dir.to_path_buf());
    }
    if let Some(dir) = dirs.runtime_dir() {
      base.insert("runtime", dir.to_path_buf());
    }
    if let Some(dir) = dirs.state_dir() {
      base.insert("state", dir.to_path_buf());
    }
  }

  let mut user: HashMap<&'static str, PathBuf> = HashMap::new();

  user.insert("home", USER_DIRS.home_dir().to_path_buf());
  if let Some(dir) = USER_DIRS.audio_dir() {
    user.insert("audio", dir.to_path_buf());
  }
  if let Some(dir) = USER_DIRS.desktop_dir() {
    user.insert("desktop", dir.to_path_buf());
  }
  if let Some(dir) = USER_DIRS.document_dir() {
    user.insert("document", dir.to_path_buf());
  }
  if let Some(dir) = USER_DIRS.download_dir() {
    user.insert("download", dir.to_path_buf());
  }
  if let Some(dir) = USER_DIRS.font_dir() {
    user.insert("font", dir.to_path_buf());
  }
  if let Some(dir) = USER_DIRS.picture_dir() {
    user.insert("picture", dir.to_path_buf());
  }
  if let Some(dir) = USER_DIRS.public_dir() {
    user.insert("public", dir.to_path_buf());
  }
  if let Some(dir) = USER_DIRS.template_dir() {
    user.insert("template", dir.to_path_buf());
  }
  if let Some(dir) = USER_DIRS.video_dir() {
    user.insert("video", dir.to_path_buf());
  }

  DirectoryPrameters { base, user }
});

#[derive(Serialize)]
struct CompleteParameters<'a, T: Serialize> {
  #[serde(flatten)]
  pub parameters: &'a T,
  pub env: &'a HashMap<String, String>,
  pub os: &'a str,
  pub whoami: &'static WhoamiPrameters,
  pub dirs: &'static DirectoryPrameters,
}

pub fn render(template: &str, parameters: &impl Serialize) -> Result<String, Error> {
  let complete = CompleteParameters {
    parameters,
    env: &ENV,
    whoami: &WHOAMI_PRAMETERS,
    os: &helpers::os::OS.to_string().to_ascii_lowercase(),
    dirs: &DIRECTORY_PRAMETERS,
  };

  HANDLEBARS.render_template(template, &complete).map_err(Error::RenderingTemplate)
}

#[cfg(test)]
mod test;
