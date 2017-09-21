#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![warn(unused_extern_crates)]

//! Yet another implementation of Mison JSON parser for Rust.

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate itertools;
extern crate num;
extern crate simd;

#[cfg(test)]
#[macro_use]
extern crate maplit;

pub mod bit;
pub mod errors;
pub mod index_builder;
pub mod parser;
pub mod query;
