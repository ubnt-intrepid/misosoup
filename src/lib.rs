//! Yet another implementation of Mison JSON parser for Rust.

#![warn(
    missing_debug_implementations, //
    rust_2018_idioms,
    unused,
    unsafe_code,
)]

#[macro_use]
extern crate error_chain;

pub mod bit;
pub mod errors;
pub mod index_builder;
pub mod parser;
pub mod pattern_tree;
pub mod query;
pub mod query_parser;
pub mod value;
