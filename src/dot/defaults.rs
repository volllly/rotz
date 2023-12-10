use std::{collections::HashMap, fs, path::Path};

use miette::Report;
use tap::Pipe;
#[cfg(feature = "profiling")]
use tracing::instrument;
use walkdir::WalkDir;
use wax::Pattern;

use super::Error;
use crate::{helpers, FileFormat, FILE_EXTENSIONS_GLOB};

#[derive(Debug)]
pub struct Defaults(HashMap<String, (String, FileFormat)>);

impl Defaults {
  pub fn for_path(&self, path: impl AsRef<str>) -> Option<&(String, FileFormat)> {
    for path in Path::new(path.as_ref()).ancestors() {
      if let Some(defaults) = self.0.get(helpers::absolutize_virtually(path).unwrap().as_str()) {
        return Some(defaults);
      }
    }

    None
  }

  #[cfg_attr(feature = "profiling", instrument)]
  pub fn from_path(dotfiles_path: &Path) -> Result<Defaults, Error> {
    let defaults = helpers::glob_from_vec(&["**".to_owned()], format!("/{{dots,defaults}}.{FILE_EXTENSIONS_GLOB}").as_str().pipe(Some)).unwrap();

    let paths = WalkDir::new(dotfiles_path).into_iter().collect::<Result<Vec<_>, _>>().map_err(Error::WalkingDotfiles)?;

    let absolutized = paths
      .into_iter()
      .filter(|e| !e.file_type().is_dir())
      .map(|d| {
        let path = d.path().strip_prefix(dotfiles_path).map(Path::to_path_buf).map_err(Error::PathStrip)?;
        let absolutized = helpers::absolutize_virtually(&path).map_err(|e| Error::ParseName(path.to_string_lossy().to_string(), e))?;
        let absolutized_dir = helpers::absolutize_virtually(path.parent().unwrap()).map_err(|e| Error::ParseName(path.to_string_lossy().to_string(), e))?;
        Ok::<_, Error>((absolutized, absolutized_dir, path))
      })
      .collect::<Result<Vec<_>, _>>()?;

    absolutized
      .into_iter()
      .filter(|e| defaults.is_match(e.0.as_str()))
      .map(|e| (e.1, e.2))
      .map(|e| {
        if e.1.file_name().unwrap().to_string_lossy().starts_with("dots.") {
          let path = e.1.to_string_lossy().to_string();
          println!(
            "Warning: {:?}",
            Report::new(Error::DotsDeprecated(
              e.1.extension().unwrap().to_string_lossy().to_string(),
              (path.rfind("dots").unwrap(), "dots".len()).into(),
              path
            ))
          );
        }

        (
          e.0,
          (
            fs::read_to_string(dotfiles_path.join(&e.1)).map_err(|err| Error::ReadingDot(e.1.clone(), err))?,
            FileFormat::try_from(e.1.as_path()).unwrap(),
          ),
        )
          .pipe(Ok)
      })
      .collect::<Result<HashMap<_, _>, _>>()
      .map(Defaults)
  }
}
