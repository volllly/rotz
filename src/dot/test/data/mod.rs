mod filters;
mod structure;

#[macro_export]
macro_rules! parse {
  ($format:literal, $engine:expr, $parameters:expr) => {
    $crate::dot::from_str_with_defaults(
      std::fs::read_to_string(std::path::Path::new(file!()).parent().unwrap().join(format!("dot.{}", $format)))
        .unwrap()
        .as_str(),
      $crate::FileFormat::try_from($format).unwrap(),
      None,
      $engine,
      $parameters,
    )
    .unwrap()
  };
}
