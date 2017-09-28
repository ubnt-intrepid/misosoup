extern crate mison;

use mison::query::QueryTree;
use mison::query_parser::QueryParser;
use mison::index_builder::IndexBuilder;

#[cfg(feature = "avx-accel")]
use mison::index_builder::backend::Avx2Backend as Backend;
#[cfg(not(feature = "avx-accel"))]
use mison::index_builder::backend::FallbackBackend as Backend;

fn main() {
    let mut tree = QueryTree::default();
    tree.add_path("$.foo").unwrap();
    tree.add_path("$.baz.hoge").unwrap();

    let index_builder = IndexBuilder::new(Backend::default(), tree.max_level());
    let parser = QueryParser::new(index_builder, tree);

    let input = r#"{ "foo": "bar", "baz": { "piyo": "fuga", "hoge": [null] } }"#;
    let result = parser.parse(input).unwrap();

    println!("{:?}", result);
}
