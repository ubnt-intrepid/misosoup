extern crate mison;

use mison::parser::Parser;
use mison::index_builder::IndexBuilder;

use mison::index_builder::backend::FallbackBackend;

fn main() {
    let level = 5;

    let index_builder = IndexBuilder::new(FallbackBackend::default(), level);
    let parser = Parser::new(index_builder);

    let input = r#"{ "foo": "bar", "baz": { "piyo": "fuga", "hoge": [null] } }"#;
    let result = parser.parse(input).unwrap();

    println!("{:#?}", result);
}
