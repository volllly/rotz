use rstest::rstest;
use speculoos::prelude::*;

use crate::config::LinkType;

#[rstest]
#[case("symbolic", LinkType::Symbolic)]
#[case("SYMBOLIC", LinkType::Symbolic)]
#[case("Symbolic", LinkType::Symbolic)]
#[case("SyMbOlIc", LinkType::Symbolic)]
#[case("hard", LinkType::Hard)]
#[case("HARD", LinkType::Hard)]
#[case("Hard", LinkType::Hard)]
#[case("HaRd", LinkType::Hard)]
fn test_linktype_case_insensitive_deserialization(#[case] input: &str, #[case] expected: LinkType) {
  let yaml = format!("link_type: {}", input);

  #[derive(serde::Deserialize, Debug)]
  struct TestConfig {
    link_type: LinkType,
  }

  let result: Result<TestConfig, _> = serde_yaml::from_str(&yaml);

  assert_that!(result).is_ok().map(|config| &config.link_type).is_equal_to(&expected);
}

#[test]
fn test_linktype_invalid_value() {
  let yaml = "link_type: invalid";

  #[derive(serde::Deserialize, Debug)]
  struct TestConfig {
    link_type: LinkType,
  }

  let result: Result<TestConfig, _> = serde_yaml::from_str(&yaml);

  assert_that!(result).is_err();

  let error_msg = result.unwrap_err().to_string();
  assert_that!(error_msg).contains("unknown variant");
  assert_that!(error_msg).contains("invalid");
  // Should suggest both capitalized and lowercase variants
  assert_that!(error_msg).contains("Symbolic");
  assert_that!(error_msg).contains("symbolic");
  assert_that!(error_msg).contains("Hard");
  assert_that!(error_msg).contains("hard");
}

#[test]
fn test_linktype_serialization_preserves_original_case() {
  // Test that serialization still uses the original enum variant names
  let symbolic = LinkType::Symbolic;
  let hard = LinkType::Hard;

  let symbolic_yaml = serde_yaml::to_string(&symbolic).unwrap();
  let hard_yaml = serde_yaml::to_string(&hard).unwrap();

  assert_that!(symbolic_yaml.trim()).is_equal_to("Symbolic");
  assert_that!(hard_yaml.trim()).is_equal_to("Hard");
}

#[test]
fn test_linktype_roundtrip_deserialization() {
  // Test that we can deserialize various cases and they all work correctly
  let test_cases = vec![
    ("symbolic", LinkType::Symbolic),
    ("SYMBOLIC", LinkType::Symbolic),
    ("Symbolic", LinkType::Symbolic),
    ("hard", LinkType::Hard),
    ("HARD", LinkType::Hard),
    ("Hard", LinkType::Hard),
  ];

  for (input, expected) in test_cases {
    let yaml = format!("link_type: {}", input);

    #[derive(serde::Deserialize, Debug)]
    struct TestConfig {
      link_type: LinkType,
    }

    let config: TestConfig = serde_yaml::from_str(&yaml).unwrap_or_else(|e| panic!("Failed to deserialize '{}': {}", input, e));

    assert_that!(config.link_type).is_equal_to(&expected);

    // Verify the deserialized value behaves correctly
    match expected {
      LinkType::Symbolic => assert_that!(config.link_type.is_symbolic()).is_true(),
      LinkType::Hard => assert_that!(config.link_type.is_hard()).is_true(),
    }
  }
}
