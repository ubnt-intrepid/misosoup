#![feature(link_llvm_intrinsics, simd_ffi)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![warn(unused_extern_crates)]

//! Yet another implementation of Mison JSON parser for Rust.

#[macro_use]
extern crate error_chain;
extern crate num;

#[cfg(feature = "simd-accel")]
extern crate simd;

#[cfg(all(feature = "avx-accel", target_arch = "x86_64"))]
extern crate stdsimd;

#[cfg(test)]
#[macro_use]
extern crate maplit;

pub mod bit;
pub mod errors;
pub mod index_builder;
pub mod parser;
pub mod query;
