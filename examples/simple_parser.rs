extern crate mison;

use mison::parser::Parser;
use mison::index_builder::IndexBuilder;

#[cfg(feature = "avx-accel")]
use mison::index_builder::backend::Avx2Backend as Backend;
#[cfg(not(feature = "avx-accel"))]
use mison::index_builder::backend::FallbackBackend as Backend;

fn main() {
    let level = 5;

    let index_builder = IndexBuilder::new(Backend::default(), level);
    let parser = Parser::new(index_builder);

    let input = r#"{ "foo": "bar", "baz": { "piyo": "fuga", "hoge": [null] } }"#;
    let result = parser.parse(input).unwrap();

    println!("{:#?}", result);
}
