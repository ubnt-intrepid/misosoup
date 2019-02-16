# `mison-rs` [![Build Status](https://travis-ci.org/ubnt-intrepid/mison-rs.svg?branch=master)](https://travis-ci.org/ubnt-intrepid/mison-rs)

A JSON parser for Rust, based on Mison.

This project is not production ready.

## Examples

Simple parser:

```rust
// src/main.rs

extern crate mison;

use mison::parser::Parser;
use mison::index_builder::IndexBuilder;
use mison::index_builder::backend::Avx2Backend;

fn main() {
    let level = 5;

    let index_builder = IndexBuilder::new(AvxBackend::default(), level);
    let parser = Parser::new(index_builder);

    let input = r#"{ "foo": "bar", "baz": { "piyo": "fuga", "hoge": [null] } }"#;
    let result = parser.parse(input).unwrap();

    println!("{:#?}", result);
}
```

```command
$ RUSTFLAGS="-C target-cpu=native" cargo +nightly run
{
    "foo": "bar",
    "baz": {
        "piyo": "fuga",
        "hoge": [
            null
        ]
    }
}
```

Query parser:

```rust
extern crate mison;

use mison::query::QueryTree;
use mison::query_parser::QueryParser;
use mison::index_builder::IndexBuilder;

use mison::index_builder::backend::AvxBackend;

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
```

```command
$ RUSTFLAGS="-C target-cpu=native" cargo +nightly run
[Some("\"bar\""), Some("[null]")]
```

## TODOs
- [ ] array query (`"$.foo[0].bar"`)
- [ ] Speculative parsing

## License
MIT/Apache 2.0
