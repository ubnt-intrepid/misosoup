#![feature(test)]

extern crate mison;
extern crate serde_json;
extern crate test;

use mison::parser::Parser;
use mison::index_builder::IndexBuilder;
use mison::index_builder::backend::FallbackBackend;
#[cfg(feature = "simd-accel")]
use mison::index_builder::backend::Sse2Backend;

const INPUT: &str = r#"{
    "f1": 10,
    "f2": {
        "e1": true,
        "e2": "hoge",
        "e3": {
            "d1": "The quick brown fox jumps over the lazy dog.",
            "d2": 100.2
        }
    },
    "f3": {
        "e3": null
    }
}"#;

#[bench]
fn bench_serde_json(b: &mut test::Bencher) {
    b.iter(|| {
        let _: serde_json::Value = serde_json::from_str(INPUT).unwrap();
    });
}

#[bench]
fn bench_mison(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::<Sse2Backend>::default();
    let parser = Parser::new(index_builder);

    b.iter(|| {
        let _ = parser.parse(INPUT, 3).unwrap();
    });
}

#[bench]
fn bench_mison_index_builder(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::<FallbackBackend>::default();

    b.iter(|| {
        let _ = index_builder.build(INPUT.as_bytes(), 3).unwrap();
    });
}

#[bench]
#[cfg(feature = "simd-accel")]
fn bench_mison_index_builder_sse2(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::<Sse2Backend>::default();

    b.iter(|| {
        let _ = index_builder.build(INPUT.as_bytes(), 3).unwrap();
    });
}
