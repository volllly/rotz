use std::str::FromStr;

use chumsky::{Parser, error::Simple, prelude::*};
#[cfg(test)]
use fake::Dummy;
use strum::EnumString;

use crate::helpers::os;
use crate::templating::{Engine, Parameters};

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
pub(super) struct Filter {
  pub os: os::Os,
  pub attributes: Vec<Attribute>,
}

fn string() -> impl Parser<char, String, Error = Simple<char>> {
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
        just('u').ignore_then(
          chumsky::prelude::filter(|c: &char| c.is_ascii_hexdigit())
            .repeated()
            .exactly(4)
            .collect::<String>()
            .validate(|digits, span, emit| {
              char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap_or_else(|| {
                emit(Simple::custom(span, "invalid unicode character"));
                '\u{FFFD}' // unicode replacement character
              })
            }),
        ),
      ),
  );

  just('"')
    .ignore_then(chumsky::prelude::filter(|c| *c != '\\' && *c != '"').or(escape).repeated())
    .then_ignore(just('"'))
    .collect::<String>()
}

fn operator() -> impl Parser<char, Operator, Error = Simple<char>> {
  just("=")
    .map(|_| Operator::Eq)
    .or(just("$=").map(|_| Operator::EndsWith))
    .or(just("^=").map(|_| Operator::StartsWith))
    .or(just("*=").map(|_| Operator::Contains))
    .or(just("!=").map(|_| Operator::NotEq))
}
fn path() -> impl Parser<char, String, Error = Simple<char>> {
  text::ident().separated_by(just('.')).map(|k| k.join("."))
}
fn attribute() -> impl Parser<char, Attribute, Error = Simple<char>> {
  path().then(operator().padded()).then(string()).map(|((key, operator), value)| Attribute { key, operator, value })
}
fn attributes() -> impl Parser<char, Vec<Attribute>, Error = Simple<char>> {
  attribute().separated_by(just(',').padded()).delimited_by(just('['), just(']')).or(empty().map(|()| vec![]))
}
fn os() -> impl Parser<char, os::Os, Error = Simple<char>> {
  text::ident().try_map(|i: String, span| os::Os::try_from(i.as_str()).map_err(|e| Simple::custom(span, e)))
}
fn filter() -> impl Parser<char, Filter, Error = Simple<char>> {
  os().then(attributes()).map(|(os, attributes)| Filter { os, attributes })
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(Dummy))]
pub struct Filters(Vec<Filter>);

impl FromStr for Filters {
  type Err = Vec<chumsky::error::Simple<char>>;
  fn from_str(s: &str) -> Result<Filters, Self::Err> {
    filter().separated_by(just('|').padded()).then_ignore(end()).parse(s).map(Filters)
  }
}

impl Filters {
  pub fn applies(&self, engine: &Engine, parameters: &Parameters) -> bool {
    self.0.iter().any(|filter| {
      (filter.os == os::Os::Global || os::OS == filter.os)
        && filter.attributes.iter().all(|attribute| {
          let value = engine.render(&format!("{{ {} }}", &attribute.key), parameters).unwrap();
          match attribute.operator {
            Operator::Eq => value == attribute.value,
            Operator::StartsWith => value.starts_with(&attribute.value),
            Operator::EndsWith => value.ends_with(&attribute.value),
            Operator::Contains => value.contains(&attribute.value),
            Operator::NotEq => value != attribute.value,
          }
        })
    })
  }
}

#[cfg(test)]
mod test {
  use super::{Attribute, Filter, Filters, Operator};
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
  #[case("[test=\"value\",test=\"value\"]", vec![Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() }, Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() }])]
  #[case("[test=\"value\", test=\"value\"]", vec![Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() }, Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() }])]
  #[case("[test=\"value\" , test=\"value\"]", vec![Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() }, Attribute { key: "test".to_owned(), operator: Operator::Eq, value: "value".to_owned() }])]
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
  #[case("windows", Filter { os: Os::Windows, attributes: vec![] })]
  #[case("windows[]", Filter { os: Os::Windows, attributes: vec![] })]
  #[case("global[test=\"some\", test=\"some\"]", Filter { os: Os::Global, attributes: vec![
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
  #[case("linux[whoami.distribution^=\"some\"]", Filter { os: Os::Linux, attributes: vec![
    Attribute {
      key: String::from("whoami.distribution"),
      operator: Operator::StartsWith,
      value: String::from("some")
      }
  ]})]
  fn filter_parser(#[case] from: &str, #[case] expected: Filter) {
    let parsed = super::filter().then_ignore(end()).parse(from).unwrap();
    assert_that!(parsed).named(from).is_equal_to(expected);
  }

  #[rstest]
  #[case("windows", vec![Filter { os: Os::Windows, attributes: vec![] }])]
  #[case("windows|linux", vec![Filter { os: Os::Windows, attributes: vec![] }, Filter { os: Os::Linux, attributes: vec![] }])]
  #[case("darwin|linux", vec![Filter { os: Os::Darwin, attributes: vec![] }, Filter { os: Os::Linux, attributes: vec![] }])]
  #[case("linux[whoami.distribution^=\"some\"]", vec![Filter { os: Os::Linux, attributes: vec![
  Attribute {
    key: String::from("whoami.distribution"),
    operator: Operator::StartsWith,
    value: String::from("some")
    }
]}])]
  #[case("windows[whoami.distribution$=\"some\", test=\"other\"]", vec![Filter { os: Os::Windows, attributes: vec![
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
  #[case("global[whoami.distribution^=\"some\"]|windows[whoami.distribution$=\"some\", test=\"other\"]", vec![Filter { os: Os::Global, attributes: vec![
  Attribute {
    key: String::from("whoami.distribution"),
    operator: Operator::StartsWith,
    value: String::from("some")
    }
] },Filter { os: Os::Windows, attributes: vec![
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
    use std::str::FromStr;

    let parsed = Filters::from_str(from).unwrap();

    assert_that!(parsed).named(from).is_equal_to(Filters(filter));
  }
}
