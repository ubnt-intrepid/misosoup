extern crate mison;

use mison::parser::Parser;
use mison::index_builder::IndexBuilder;
use mison::index_builder::backend::Sse2Backend;

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
    let index_builder = IndexBuilder::<Sse2Backend>::default();
    let parser = Parser::new(index_builder);
    let parsed = parser.parse(input, 3).unwrap();
    println!("{:?}", parsed);
}
