# `misosoup`

[![Build Status](https://travis-ci.org/ubnt-intrepid/mison-rs.svg?branch=master)](https://travis-ci.org/ubnt-intrepid/mison-rs)

An experimental implementation of Mison JSON parser, written in Rust. 

> This project is experimental and DO NOT use for production use. 

## Examples

Simple parser:

```rust
use misosoup::parser::Parser;
use misosoup::index_builder::IndexBuilder;
use misosoup::index_builder::backend::Avx2Backend;

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
use misosoup::query::QueryTree;
use misosoup::query_parser::QueryParser;
use misosoup::index_builder::IndexBuilder;
use misosoup::index_builder::backend::AvxBackend;

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

This project is licensed under either of

* MIT license ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
