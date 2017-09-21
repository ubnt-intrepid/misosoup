#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![warn(unused_extern_crates)]

//! Yet another implementation of Mison JSON parser for Rust.

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate itertools;
#[cfg(test)]
#[macro_use]
extern crate maplit;
extern crate num;
extern crate simd;

pub mod bit;
pub mod errors;
pub mod index_builder;
pub mod query;
