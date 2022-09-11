#[macro_export]
macro_rules! parse {
  ($format:literal) => {
    $crate::dot::from_str_with_defaults(
      std::fs::read_to_string(std::path::Path::new(file!()).parent().unwrap().join(format!("dot.{}", $format)))
        .unwrap()
        .as_str(),
      $crate::FileFormat::try_from($format).unwrap(),
      None,
    )
    .unwrap()
  };
}

mod s01;
mod s02;
mod s03;
