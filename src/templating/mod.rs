use std::{collections::HashMap, fmt::Debug, path::PathBuf};

use directories::BaseDirs;
use handlebars::{Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError, Renderable, ScopedJson};
use itertools::Itertools;
use miette::Diagnostic;
use once_cell::sync::Lazy;
use serde::Serialize;
use tap::{Conv, Pipe};
#[cfg(feature = "profiling")]
use tracing::instrument;
use velcro::hash_map;

use crate::{
  cli::Cli,
  config::Config,
  helpers::{self, os},
  USER_DIRS,
};

pub static ENV: Lazy<HashMap<String, String>> = Lazy::new(|| std::env::vars().collect());

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum Error {
  #[error("Could not render templeate")]
  #[diagnostic(code(template::render))]
  RenderingTemplate(#[source] handlebars::RenderError),
}

#[derive(Serialize, Debug)]
pub struct Parameters<'a> {
  pub config: &'a Config,
  pub name: &'a str,
}

#[derive(Serialize, Debug)]
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

#[derive(Serialize, Debug)]
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

#[derive(Serialize, Debug)]
struct CompleteParameters<'a, T: Serialize> {
  #[serde(flatten)]
  pub parameters: &'a T,
  pub env: &'a HashMap<String, String>,
  pub os: &'a str,
  pub whoami: &'static WhoamiPrameters,
  pub dirs: &'static DirectoryPrameters,
}

pub(crate) struct Engine<'a>(Handlebars<'a>);

impl<'b> Engine<'b> {
  #[cfg_attr(feature = "profiling", instrument)]
  pub fn new<'a>(config: &'a Config, cli: &'a Cli) -> Engine<'b> {
    let mut hb = handlebars_misc_helpers::new_hbs::<'b>();
    hb.set_strict_mode(false);

    hb.register_helper("windows", WindowsHelper.conv::<Box<_>>());
    hb.register_helper("linux", LinuxHelper.conv::<Box<_>>());
    hb.register_helper("darwin", DarwinHelper.conv::<Box<_>>());

    hb.register_helper(
      "eval",
      EvalHelper {
        shell_command: config.shell_command.as_ref().cloned(),
        dry_run: cli.dry_run,
      }
      .pipe(Box::new),
    );

    Self(hb)
  }

  #[cfg_attr(feature = "profiling", instrument(skip(self)))]
  pub fn render(&self, template: &str, parameters: &(impl Serialize + Debug)) -> Result<String, Error> {
    let complete = CompleteParameters {
      parameters,
      env: &ENV,
      whoami: &WHOAMI_PRAMETERS,
      os: &helpers::os::OS.to_string().to_ascii_lowercase(),
      dirs: &DIRECTORY_PRAMETERS,
    };

    self.render_template(template, &complete).map_err(Error::RenderingTemplate)
  }

  #[cfg_attr(feature = "profiling", instrument(skip(self)))]
  pub fn render_template(&self, template_string: &str, data: &(impl Serialize + Debug)) -> Result<String, RenderError> {
    self.0.render_template(template_string, data)
  }
}

pub struct WindowsHelper;

impl HelperDef for WindowsHelper {
  #[cfg_attr(feature = "profiling", instrument(skip(self, out)))]
  fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, r: &'reg Handlebars<'reg>, ctx: &'rc Context, rc: &mut RenderContext<'reg, 'rc>, out: &mut dyn Output) -> HelperResult {
    if os::OS.is_windows() { h.template() } else { h.inverse() }.map(|t| t.render(r, ctx, rc, out)).map_or(Ok(()), |r| r)
  }
}

pub struct LinuxHelper;

impl HelperDef for LinuxHelper {
  #[cfg_attr(feature = "profiling", instrument(skip(self, out)))]
  fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, r: &'reg Handlebars<'reg>, ctx: &'rc Context, rc: &mut RenderContext<'reg, 'rc>, out: &mut dyn Output) -> HelperResult {
    if os::OS.is_linux() { h.template() } else { h.inverse() }.map(|t| t.render(r, ctx, rc, out)).map_or(Ok(()), |r| r)
  }
}

pub struct DarwinHelper;

impl HelperDef for DarwinHelper {
  #[cfg_attr(feature = "profiling", instrument(skip(self, out)))]
  fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, r: &'reg Handlebars<'reg>, ctx: &'rc Context, rc: &mut RenderContext<'reg, 'rc>, out: &mut dyn Output) -> HelperResult {
    if os::OS.is_darwin() { h.template() } else { h.inverse() }.map(|t| t.render(r, ctx, rc, out)).map_or(Ok(()), |r| r)
  }
}

pub struct EvalHelper {
  shell_command: Option<String>,
  dry_run: bool,
}

impl HelperDef for EvalHelper {
  #[cfg_attr(feature = "profiling", instrument(skip(self)))]
  fn call_inner<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, r: &'reg Handlebars<'reg>, _: &'rc Context, _: &mut RenderContext<'reg, 'rc>) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
    let cmd = h
      .param(0)
      .ok_or_else(|| RenderError::new("Param not found for helper \"eval\""))?
      .value()
      .as_str()
      .ok_or_else(|| RenderError::new("Param needs to be a string \"eval\""))?;

    if self.dry_run {
      format!("{{{{ eval \"{cmd}\" }}}}").conv::<handlebars::JsonValue>().conv::<handlebars::ScopedJson>().pipe(Ok)
    } else {
      let cmd = if let Some(shell_command) = self.shell_command.as_ref() {
        r.render_template(shell_command, &hash_map! { "cmd": &cmd })
          .map_err(|err| RenderError::from_error("Could not render shell command", err))?
      } else {
        cmd.to_owned()
      };

      let cmd = shellwords::split(&cmd).map_err(|e| RenderError::from_error("Could not parse eval command", e))?;

      match helpers::run_command(&cmd[0], &cmd[1..], true, false) {
        Err(err) => RenderError::from_error("Eval command did not run successfully", err).pipe(Err),
        Ok(result) => result.trim().conv::<handlebars::JsonValue>().conv::<handlebars::ScopedJson>().pipe(Ok),
      }
    }
  }
}

#[cfg(test)]
pub mod test;
