use misosoup::index_builder::IndexBuilder;
use misosoup::query::QueryTree;
use misosoup::query_parser::{QueryParser, QueryParserMode};
use misosoup::index_builder::backend::FallbackBackend;

fn main() {
    let mut tree = QueryTree::default();
    tree.add_path("$.foo").unwrap();
    tree.add_path("$.baz.hoge").unwrap();

    let index_builder = IndexBuilder::new(FallbackBackend::default(), tree.max_level());
    let parser = QueryParser::new(index_builder, tree);

    let input = r#"{ "foo": "bar", "baz": { "piyo": "fuga", "hoge": [null] } }"#;
    let result = parser.parse(input, QueryParserMode::Basic).unwrap();

    println!("{:?}", result);
}
