//! Definition of index builder and structural indices

pub mod backend;
mod builder;
mod index;

pub use self::builder::IndexBuilder;
pub use self::index::StructuralIndex;
