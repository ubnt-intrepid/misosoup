#![feature(link_llvm_intrinsics, simd_ffi)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![warn(unused_extern_crates)]

//! Yet another implementation of Mison JSON parser for Rust.

#[macro_use]
extern crate error_chain;
extern crate fnv;
extern crate num;

#[cfg(all(any(feature = "simd-accel", feature = "avx-accel"), target_arch = "x86_64"))]
extern crate stdsimd;

pub mod value;
pub mod bit;
pub mod errors;
pub mod index_builder;
pub mod parser;
pub mod pattern_tree;
pub mod query;
pub mod query_parser;
