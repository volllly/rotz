use super::{Attribute, Filter, Operator};
use crate::os::Os;
use rstest::rstest;
use speculoos::assert_that;

#[rstest]
#[case("windows", vec![Filter { os: Os::Windows, attributes: vec![] }])]
#[case("windows|linux", vec![Filter { os: Os::Windows, attributes: vec![] }, Filter { os: Os::Linux, attributes: vec![] }])]
#[case("darwin|linux", vec![Filter { os: Os::Darwin, attributes: vec![] }, Filter { os: Os::Linux, attributes: vec![] }])]
#[case("linux[whoami.distribution^=\"some\"]", vec![Filter { os: Os::Windows, attributes: vec![
  Attribute {
    key: String::from("whoami.distribution"),
    operator: Operator::StartsWith,
    value: String::from("some")
    }
]}])]
#[case("linux[whoami.distribution&=\"some\", test=\"other\"]", vec![Filter { os: Os::Windows, attributes: vec![
  Attribute {
    key: String::from("whoami.distribution"),
    operator: Operator::EndsWith,
    value: String::from("some")
    },
    Attribute {
      key: String::from("test"),
      operator: Operator::Eq,
      value: String::from("other")
      }
] }])]
fn filter_deserialization(#[case] from: &str, #[case] filter: Vec<Filter>) {
  let parsed = Filter::from(from).unwrap();

  assert_that!(parsed).is_equal_to(&filter);
}
