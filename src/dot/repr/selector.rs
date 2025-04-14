use std::str::FromStr;

use chumsky::{Parser, prelude::*};
#[cfg(test)]
use fake::Dummy;
use miette::{Diagnostic, LabeledSpan};
use strum::EnumString;
use tap::Pipe;

use crate::helpers::{MultipleErrors, os};
use crate::templating::{self, Engine, Parameters};
use thiserror::Error;

#[derive(Debug, EnumString, Hash, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(Dummy))]
pub(super) enum Operator {
  #[strum(serialize = "=")]
  Eq,
  #[strum(serialize = "^=")]
  StartsWith,
  #[strum(serialize = "$=")]
  EndsWith,
  #[strum(serialize = "*=")]
  Contains,
  #[strum(serialize = "!=")]
  NotEq,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(Dummy))]
pub(super) struct Attribute {
  key: String,
  operator: Operator,
  value: String,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(Dummy))]
pub(super) struct Selector {
  pub os: os::Os,
  pub attributes: Vec<Attribute>,
}

fn string<'src>() -> impl Parser<'src, &'src str, String, extra::Err<Rich<'src, char>>> {
  let escape = just('\\').ignore_then(
    just('\\')
      .or(just('/'))
      .or(just('"'))
      .or(just('b').to('\x08'))
      .or(just('f').to('\x0C'))
      .or(just('n').to('\n'))
      .or(just('r').to('\r'))
      .or(just('t').to('\t'))
      .or(
        just('u').ignore_then(any().filter(|c: &char| c.is_ascii_hexdigit()).repeated().exactly(4).collect::<String>().validate(|digits, e, emitter| {
          char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap_or_else(|| {
            emitter.emit(Rich::custom(e.span(), "invalid unicode character"));
            '\u{FFFD}' // unicode replacement character
          })
        })),
      ),
  );

  just('"')
    .ignore_then(any().filter(|c| *c != '\\' && *c != '"').or(escape).repeated().collect::<Vec<_>>())
    .then_ignore(just('"'))
    .map(|s| s.into_iter().collect::<String>())
}

fn operator<'src>() -> impl Parser<'src, &'src str, Operator, extra::Err<Rich<'src, char>>> {
  just("=")
    .map(|_| Operator::Eq)
    .or(just("$=").map(|_| Operator::EndsWith))
    .or(just("^=").map(|_| Operator::StartsWith))
    .or(just("*=").map(|_| Operator::Contains))
    .or(just("!=").map(|_| Operator::NotEq))
}
fn path<'src>() -> impl Parser<'src, &'src str, String, extra::Err<Rich<'src, char>>> {
  text::ident().separated_by(just('.')).at_least(1).collect::<Vec<&str>>().map(|k| k.join("."))
}
fn attribute<'src>() -> impl Parser<'src, &'src str, Attribute, extra::Err<Rich<'src, char>>> {
  path().then(operator().padded()).then(string()).map(|((key, operator), value)| Attribute { key, operator, value })
}
fn attributes<'src>() -> impl Parser<'src, &'src str, Vec<Attribute>, extra::Err<Rich<'src, char>>> {
  attribute().delimited_by(just('['), just(']')).padded().repeated().collect::<Vec<_>>()
}
fn os<'src>() -> impl Parser<'src, &'src str, os::Os, extra::Err<Rich<'src, char>>> {
  text::ident().try_map(|i: &str, span| os::Os::try_from(i).map_err(|e| Rich::custom(span, e)))
}
fn selector<'src>() -> impl Parser<'src, &'src str, Selector, extra::Err<Rich<'src, char>>> {
  os().then(attributes()).map(|(os, attributes)| Selector { os, attributes })
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(Dummy))]
pub struct Selectors(Vec<Selector>);

impl FromStr for Selectors {
  type Err = MultipleErrors;
  fn from_str(s: &str) -> Result<Selectors, Self::Err> {
    selector()
      .separated_by(just('|').padded())
      .collect::<Vec<_>>()
      .then_ignore(end())
      .try_map(|selectors, span| {
        let selectors = Selectors(selectors);
        if selectors.is_global() && selectors.0.len() > 1 {
          Rich::custom(span, "global can not be mixed with an operating system").pipe(Err)
        } else {
          selectors.pipe(Ok)
        }
      })
      .parse(s)
      .into_result()
      .map_err(|e| MultipleErrors::from_chumsky(s, e))
  }
}

impl Selectors {
  pub fn is_global(&self) -> bool {
    self.0.iter().any(|f| f.os == os::Os::Global)
  }

  pub fn applies(&self, engine: &Engine, parameters: &Parameters) -> Result<bool, Vec<templating::Error>> {
    let mut errors = Vec::<templating::Error>::new();
    let mut applies = false;
    for selector in &self.0 {
      if selector.os.is_global() || os::OS == selector.os {
        let mut all = true;
        for attribute in &selector.attributes {
          let value = match engine.render(&format!("{{{{ {} }}}}", &attribute.key), parameters) {
            Ok(v) => v,
            Err(e) => {
              errors.push(e);
              continue;
            }
          };

          if !match attribute.operator {
            Operator::Eq => value == attribute.value,
            Operator::StartsWith => value.starts_with(&attribute.value),
            Operator::EndsWith => value.ends_with(&attribute.value),
            Operator::Contains => value.contains(&attribute.value),
            Operator::NotEq => value != attribute.value,
          } {
            all = false;
          }
        }
        if all {
          applies = true;
        }
      }
    }

    if !errors.is_empty() {
      return Err(errors);
    }

    Ok(applies)
  }
}

#[derive(Error, Debug, Diagnostic)]
#[error("{reason}")]
#[diagnostic(code(parsing::selector::error))]
struct SelectorError {
  #[source_code]
  src: String,
  #[label(collection, "error happened here")]
  labels: Vec<LabeledSpan>,
  reason: String,
}

impl MultipleErrors {
  fn from_chumsky(selector: &str, errors: Vec<Rich<char>>) -> Self {
    MultipleErrors::from(
      errors
        .into_iter()
        .map(|e| {
          let (reason, labels) = match e.reason() {
            chumsky::error::RichReason::ExpectedFound { expected, found } => (
              found.map_or_else(|| "Selector ended unexpectedly".to_owned(), |f| format!("unexpected input: {f:?}")),
              expected
                .iter()
                .map(|p| LabeledSpan::new_with_span(Some(format!("expected one of: {p}")), e.span().into_range()))
                .collect(),
            ),
            chumsky::error::RichReason::Custom(c) => (c.clone(), vec![LabeledSpan::new_with_span(None, e.span().into_range())]),
          };

          SelectorError {
            src: selector.to_owned(),
            reason,
            labels,
          }
        })
        .collect::<Vec<SelectorError>>(),
    )
  }
}

#[cfg(test)]
mod test {
  use super::{Attribute, Operator, Selector, Selectors};
  use crate::helpers::os;
  use crate::os::Os;
  use chumsky::{Parser, prelude::end};
  use rstest::rstest;
  use speculoos::assert_that;

  #[rstest]
  #[case("\"test\"", "test")]
  #[case("\"tes444t\"", "tes444t")]
  #[case("\"tes44 sf sdf \\\"sdf\\n dfg \\t g \\b \\f \\r \\/ \\\\4t\"", "tes44 sf sdf \"sdf\n dfg \t g \x08 \x0C \r / \\4t")]
  fn string_parser(#[case] from: &str, #[case] expected: &str) {
    let parsed = super::string().then_ignore(end()).parse(from).unwrap();
    assert_that!(parsed.as_str()).is_equal_to(expected);
  }

  #[rstest]
  #[case("test")]
  #[case("test.tt")]
  #[case("test.t04.e")]
  fn path_parser(#[case] from: &str) {
    let parsed = super::path().then_ignore(end()).parse(from).unwrap();
    assert_that!(parsed.as_str()).is_equal_to(from);
  }

  #[rstest]
  #[case("=", Operator::Eq)]
  #[case("^=", Operator::StartsWith)]
  #[case("$=", Operator::EndsWith)]
  #[case("*=", Operator::Contains)]
  #[case("!=", Operator::NotEq)]
  fn operator_parser(#[case] from: &str, #[case] expected: Operator) {
    let parsed = super::operator().then_ignore(end()).parse(from).unwrap();
    assert_that!(parsed).is_equal_to(expected);
  }

  #[rstest]
  #[case("test=\"value\"", Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() })]
  #[case("test.test=\"value\"", Attribute { key: "test.test".to_owned(), operator: Operator::Eq, value: "value".to_owned() })]
  #[case("test =\"value\"", Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() })]
  #[case("test= \"value\"", Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() })]
  #[case("test = \"value\"", Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() })]
  #[case("test^=\"value\"", Attribute { key: "test".to_owned(), operator: Operator::StartsWith, value: "value".to_owned() })]
  #[case("test $=\"value\"", Attribute { key: "test".to_owned(), operator: Operator::EndsWith, value: "value".to_owned() })]
  #[case("test*= \"value\"", Attribute { key: "test".to_owned(), operator: Operator::Contains, value: "value".to_owned() })]
  #[case("test != \"value\"", Attribute { key: "test".to_owned(), operator: Operator::NotEq, value: "value".to_owned() })]
  fn attribute_parser(#[case] from: &str, #[case] expected: Attribute) {
    let parsed = super::attribute().then_ignore(end()).parse(from).unwrap();
    assert_that!(parsed).is_equal_to(expected);
  }

  #[rstest]
  #[case("[test=\"value\"]", vec![Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() }])]
  #[case("[test=\"value\"][test=\"value\"]", vec![Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() }, Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() }])]
  #[case("[test=\"value\"][test=\"value\"]", vec![Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() }, Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() }])]
  #[case("[test=\"value\"][test=\"value\"]", vec![Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() }, Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() }])]
  fn attributes_parser(#[case] from: &str, #[case] expected: Vec<Attribute>) {
    let parsed = super::attributes().then_ignore(end()).parse(from).unwrap();
    assert_that!(parsed).is_equal_to(expected);
  }

  #[rstest]
  #[case("windows", Os::Windows)]
  #[case("linux", Os::Linux)]
  #[case("darwin", Os::Darwin)]
  #[case("global", Os::Global)]
  fn os_parser(#[case] from: &str, #[case] expected: os::Os) {
    let parsed = super::os().then_ignore(end()).parse(from).unwrap();
    assert_that!(parsed).is_equal_to(expected);
  }

  #[rstest]
  #[case("windows", Selector { os: Os::Windows, attributes: vec![] })]
  #[case("global[test=\"some\"][test=\"some\"]", Selector { os: Os::Global, attributes: vec![
    Attribute {
      key: String::from("test"),
      operator: Operator::Eq,
      value: String::from("some")
      },
      Attribute {
        key: String::from("test"),
        operator: Operator::Eq,
        value: String::from("some")
        }
  ]})]
  #[case("linux[whoami.distribution^=\"some\"]", Selector { os: Os::Linux, attributes: vec![
    Attribute {
      key: String::from("whoami.distribution"),
      operator: Operator::StartsWith,
      value: String::from("some")
      }
  ]})]
  fn selector_parser(#[case] from: &str, #[case] expected: Selector) {
    let parsed = super::selector().then_ignore(end()).parse(from).unwrap();
    assert_that!(parsed).named(from).is_equal_to(expected);
  }

  #[rstest]
  #[case("windows", vec![Selector { os: Os::Windows, attributes: vec![] }])]
  #[case("windows|linux", vec![Selector { os: Os::Windows, attributes: vec![] }, Selector { os: Os::Linux, attributes: vec![] }])]
  #[case("darwin|linux", vec![Selector { os: Os::Darwin, attributes: vec![] }, Selector { os: Os::Linux, attributes: vec![] }])]
  #[case("linux[whoami.distribution^=\"some\"]", vec![Selector { os: Os::Linux, attributes: vec![
  Attribute {
    key: String::from("whoami.distribution"),
    operator: Operator::StartsWith,
    value: String::from("some")
    }
]}])]
  #[case("global[whoami.distribution$=\"some\"][test=\"other\"]", vec![Selector { os: Os::Global, attributes: vec![
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
  #[case("linux[whoami.distribution^=\"some\"]|windows[whoami.distribution$=\"some\"][test=\"other\"]", vec![Selector { os: Os::Linux, attributes: vec![
  Attribute {
    key: String::from("whoami.distribution"),
    operator: Operator::StartsWith,
    value: String::from("some")
    }
] },Selector { os: Os::Windows, attributes: vec![
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
  fn selector_deserialization(#[case] from: &str, #[case] selector: Vec<Selector>) {
    use std::str::FromStr;

    let parsed = Selectors::from_str(from).unwrap();

    assert_that!(parsed).named(from).is_equal_to(Selectors(selector));
  }

  #[rstest]
  #[case("windows[")]
  #[case("windows[]")]
  #[case("windows[test=]")]
  #[case("windows[test=\"test\"")]
  #[case("windows[test=\"]")]
  #[case("windows[999=\"\"]")]
  #[case("windows[999##=\"\"]")]
  #[case("windows test=\"\"]")]
  fn errors(#[case] from: &str) {
    use std::str::FromStr;

    Selectors::from_str(from).unwrap_err();
  }
}
