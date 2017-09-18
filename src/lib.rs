#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![warn(unused_extern_crates)]

//! Yet another implementation of Mison JSON parser for Rust.

extern crate itertools;
extern crate num;
extern crate simd;

#[allow(missing_docs)]
pub mod index_builder;

mod bit;
