extern crate mison;

use mison::parser::Parser;
use mison::index_builder::IndexBuilder;
use mison::index_builder::backend::FallbackBackend;

fn main() {
    let input = r#"{
        "f1": 10,
        "f2": {
            "e1": true,
            "e2": "hoge"
        },
        "f3": {
            "e3": null
        }
    }"#;
    // let input = include_str!("../benches/temp.json");
    let index_builder = IndexBuilder::new(FallbackBackend::default(), 3);
    let parser = Parser::new(index_builder);
    let parsed = parser.parse(input).unwrap();
    println!("{:#?}", parsed);
}
