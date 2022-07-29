use std::collections::HashMap;

use handlebars::Handlebars;
use miette::Diagnostic;
use once_cell::sync::Lazy;
use serde::Serialize;

use crate::{config::Config, helpers};

pub(crate) static HANDLEBARS: Lazy<Handlebars> = Lazy::new(|| {
  let mut hb = handlebars_misc_helpers::new_hbs();
  hb.set_strict_mode(false);
  hb
});

pub(crate) static ENV: Lazy<HashMap<String, String>> = Lazy::new(|| std::env::vars().collect());

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum Error {
  #[error("Could not render templeate")]
  #[diagnostic(code(template::render))]
  RenderingTemplate(#[source] handlebars::RenderError),
}

#[derive(Serialize)]
pub struct Parameters<'a> {
  pub config: &'a Config,
  pub name: &'a str,
}

#[derive(Serialize)]
struct CompleteParameters<'a, T: Serialize> {
  #[serde(flatten)]
  pub parameters: &'a T,
  pub env: &'a HashMap<String, String>,
  pub os: &'a str,
}

pub(crate) fn render(template: &str, parameters: &impl Serialize) -> Result<String, Error> {
  let complete = CompleteParameters {
    parameters,
    env: &ENV,
    os: &helpers::os::OS.to_string().to_ascii_lowercase(),
  };

  HANDLEBARS.render_template(template, &complete).map_err(Error::RenderingTemplate)
}
