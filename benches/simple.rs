#![feature(test)]

extern crate mison;
extern crate pikkr;
extern crate serde_json;
extern crate test;

#[cfg(feature = "avx-accel")]
use mison::index_builder::backend::AvxBackend;
use mison::index_builder::backend::FallbackBackend;
#[cfg(feature = "simd-accel")]
use mison::index_builder::backend::Sse2Backend;
use mison::index_builder::IndexBuilder;
use mison::parser::Parser;
use mison::query::QueryTree;
use mison::query_parser::{QueryParser, QueryParserMode};

use pikkr::Pikkr;

const INPUT: &str = include_str!("temp.json");

#[bench]
fn bench_serde_json(b: &mut test::Bencher) {
    b.iter(|| {
        let _: serde_json::Value = serde_json::from_str(INPUT).unwrap();
    });
}

#[bench]
fn bench_mison_fallback(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::new(FallbackBackend::default(), 3);
    let parser = Parser::new(index_builder);

    b.iter(|| {
        let _ = parser.parse(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_avx(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::new(AvxBackend::default(), 3);
    let parser = Parser::new(index_builder);

    b.iter(|| {
        let _ = parser.parse(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_avx_2(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::new(AvxBackend::default(), 1);
    let parser = Parser::new(index_builder);

    b.iter(|| {
        let _ = parser.parse(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_avx_3(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::new(AvxBackend::default(), 20);
    let parser = Parser::new(index_builder);

    b.iter(|| {
        let _ = parser.parse(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_avx_basic_1(b: &mut test::Bencher) {
    let mut queries = QueryTree::default();
    queries.add_path("$._id.$oid").unwrap();
    let index_builder = IndexBuilder::new(AvxBackend::default(), queries.max_level());
    let parser = QueryParser::new(index_builder, queries);

    b.iter(|| {
        let _ = parser.parse(INPUT, QueryParserMode::Basic).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_avx_basic_2(b: &mut test::Bencher) {
    let mut queries = QueryTree::default();
    queries.add_path("$._id.$oid").unwrap();
    queries.add_path("$.partners").unwrap();
    let index_builder = IndexBuilder::new(AvxBackend::default(), queries.max_level());
    let parser = QueryParser::new(index_builder, queries);

    b.iter(|| {
        let _ = parser.parse(INPUT, QueryParserMode::Basic).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_avx_basic_3(b: &mut test::Bencher) {
    let mut queries = QueryTree::default();
    queries.add_path("$.partners").unwrap();
    let index_builder = IndexBuilder::new(AvxBackend::default(), queries.max_level());
    let parser = QueryParser::new(index_builder, queries);

    b.iter(|| {
        let _ = parser.parse(INPUT, QueryParserMode::Basic).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_avx_speculative_1(b: &mut test::Bencher) {
    let mut queries = QueryTree::default();
    queries.add_path("$._id.$oid").unwrap();
    let index_builder = IndexBuilder::new(AvxBackend::default(), queries.max_level());
    let mut parser = QueryParser::new(index_builder, queries);

    // train
    parser.save_patterns(true);
    let _ = parser.parse(INPUT, QueryParserMode::Basic).unwrap();
    parser.save_patterns(false);

    b.iter(|| {
        let _ = parser.parse(INPUT, QueryParserMode::Speculative).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_avx_speculative_2(b: &mut test::Bencher) {
    let mut queries = QueryTree::default();
    queries.add_path("$._id.$oid").unwrap();
    queries.add_path("$.partners").unwrap();
    let index_builder = IndexBuilder::new(AvxBackend::default(), queries.max_level());
    let mut parser = QueryParser::new(index_builder, queries);

    // train
    parser.save_patterns(true);
    let _ = parser.parse(INPUT, QueryParserMode::Basic).unwrap();
    parser.save_patterns(false);

    b.iter(|| {
        let _ = parser.parse(INPUT, QueryParserMode::Speculative).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_avx_speculative_3(b: &mut test::Bencher) {
    let mut queries = QueryTree::default();
    queries.add_path("$.partners").unwrap();
    let index_builder = IndexBuilder::new(AvxBackend::default(), queries.max_level());
    let mut parser = QueryParser::new(index_builder, queries);

    // train
    parser.save_patterns(true);
    let _ = parser.parse(INPUT, QueryParserMode::Basic).unwrap();
    parser.save_patterns(false);

    b.iter(|| {
        let _ = parser.parse(INPUT, QueryParserMode::Speculative).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_pikkr_basic_1(b: &mut test::Bencher) {
    let mut pikkr = Pikkr::new(&["$._id.$oid"], ::std::usize::MAX).unwrap();

    b.iter(|| {
        let _ = pikkr.parse(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_pikkr_basic_2(b: &mut test::Bencher) {
    let mut pikkr = Pikkr::new(&["$._id.$oid", "$.partners"], ::std::usize::MAX).unwrap();

    b.iter(|| {
        let _ = pikkr.parse(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_pikkr_basic_3(b: &mut test::Bencher) {
    let mut pikkr = Pikkr::new(&["$.partners"], ::std::usize::MAX).unwrap();

    b.iter(|| {
        let _ = pikkr.parse(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_pikkr_speculative_1(b: &mut test::Bencher) {
    let mut pikkr = Pikkr::new(&["$._id.$oid"], 1).unwrap();
    let _ = pikkr.parse(INPUT).unwrap();

    b.iter(|| {
        let _ = pikkr.parse(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_pikkr_speculative_2(b: &mut test::Bencher) {
    let mut pikkr = Pikkr::new(&["$._id.$oid", "$.partners"], 1).unwrap();
    let _ = pikkr.parse(INPUT).unwrap();

    b.iter(|| {
        let _ = pikkr.parse(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_pikkr_speculative_3(b: &mut test::Bencher) {
    let mut pikkr = Pikkr::new(&["$.partners"], 1).unwrap();
    let _ = pikkr.parse(INPUT).unwrap();

    b.iter(|| {
        let _ = pikkr.parse(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_pikkr_index_builder(b: &mut test::Bencher) {
    use pikkr::index_builder::IndexBuilder;
    let mut index_builder = IndexBuilder::new(3);
    b.iter(|| {
        index_builder
            .build_structural_indices(INPUT.as_bytes())
            .unwrap();
    });
}

#[bench]
fn bench_mison_index_builder_fallback(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::new(FallbackBackend::default(), 3);

    b.iter(|| {
        let _ = index_builder.build(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "simd-accel")]
fn bench_mison_index_builder_sse2(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::new(Sse2Backend::default(), 3);

    b.iter(|| {
        let _ = index_builder.build(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_index_builder_avx(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::new(AvxBackend::default(), 3);

    b.iter(|| {
        let _ = index_builder.build(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_index_builder_avx_2(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::new(AvxBackend::default(), 1);

    b.iter(|| {
        let _ = index_builder.build(INPUT).unwrap();
    });
}

#[bench]
#[cfg(feature = "avx-accel")]
fn bench_mison_index_builder_avx_3(b: &mut test::Bencher) {
    let index_builder = IndexBuilder::new(AvxBackend::default(), 25);

    b.iter(|| {
        let _ = index_builder.build(INPUT).unwrap();
    });
}
