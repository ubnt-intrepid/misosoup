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

pub mod index_builder;
pub mod query;

mod bit;

#[allow(missing_docs)]
pub mod errors {
    error_chain! {
        types {
            Error, ErrorKind, ResultExt, Result;
        }

        errors {
            #[allow(missing_docs)]
            InvalidQuery {
                description("invalid query")
                display("invalid query")
            }
        }
    }
}
