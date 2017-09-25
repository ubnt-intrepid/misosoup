#![feature(test)]

extern crate mison;
extern crate pikkr;
extern crate serde_json;
extern crate test;

use mison::query::QueryTree;
use mison::parser::{Parser, QueryParser};
use mison::index_builder::IndexBuilder;
use mison::index_builder::backend::FallbackBackend;
#[cfg(feature = "simd-accel")]
use mison::index_builder::backend::Sse2Backend;
#[cfg(feature = "avx-accel")]
use mison::index_builder::backend::AvxBackend;

use pikkr::Pikkr;


const INPUT: &str = include_str!("temp.json");


#[bench]
fn bench_serde_json(b: &mut test::Bencher) {
    b.iter(|| {
        let _: serde_json::Value = serde_json::from_str(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_avx(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::<AvxBackend>::default();
    let parser = Parser::new(index_builder);

    b.iter(|| {
        let _ = parser.parse(INPUT, 3).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_avx_queried(b: &mut test::Bencher) {
    let mut queries = QueryTree::default();
    queries.add_path("$._id.$oid").unwrap();
    let index_builder = IndexBuilder::<AvxBackend>::default();
    let parser = QueryParser::new(queries, index_builder);

    b.iter(|| {
        let _ = parser.parse(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_avx_queried_2(b: &mut test::Bencher) {
    let mut queries = QueryTree::default();
    queries.add_path("$._id.$oid").unwrap();
    queries.add_path("$.partners").unwrap();
    let index_builder = IndexBuilder::<AvxBackend>::default();
    let parser = QueryParser::new(queries, index_builder);

    b.iter(|| {
        let _ = parser.parse(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_pikkr(b: &mut test::Bencher) {
    let mut pikkr = Pikkr::new(&["$._id.$oid", "$.partners"], ::std::usize::MAX).unwrap();

    b.iter(|| {
        let _ = pikkr.parse(INPUT).unwrap();
    });
}

#[bench]
fn bench_mison_index_builder_fallback(b: &mut test::Bencher) {
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

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_index_builder_avx(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::<AvxBackend>::default();

    b.iter(|| {
        let _ = index_builder.build(INPUT.as_bytes(), 3).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_index_builder_avx_2(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::<AvxBackend>::default();

    b.iter(|| {
        let _ = index_builder.build(INPUT.as_bytes(), 1).unwrap();
    });
}
