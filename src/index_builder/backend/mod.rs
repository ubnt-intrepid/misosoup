//! Definition of backends to create character bitmaps

mod sse2;

pub use self::sse2::Sse2Backend;
use super::Bitmap;


/// Represents the backend of `IndexBuilder` to create character bitmaps
pub trait Backend {
    /// Create a new bitmap from slice of bytes
    fn create_full_bitmap(&self, s: &[u8], offset: usize) -> Bitmap;

    /// Create a new bitmap from slice of bytes, whose length may be less than 64.
    fn create_partial_bitmap(&self, s: &[u8], offset: usize) -> Bitmap;
}
