use super::{Backend, Bitmap};
use packed_simd::u8x32;

#[allow(missing_docs)]
#[derive(Debug)]
pub struct AvxBackend {
    backslash: u8x32,
    quote: u8x32,
    colon: u8x32,
    comma: u8x32,
    left_brace: u8x32,
    right_brace: u8x32,
    left_bracket: u8x32,
    right_bracket: u8x32,
}

impl Default for AvxBackend {
    fn default() -> Self {
        Self {
            backslash: u8x32::splat(b'\\'),
            quote: u8x32::splat(b'"'),
            colon: u8x32::splat(b':'),
            comma: u8x32::splat(b','),
            left_brace: u8x32::splat(b'{'),
            right_brace: u8x32::splat(b'}'),
            left_bracket: u8x32::splat(b'['),
            right_bracket: u8x32::splat(b']'),
        }
    }
}

impl Backend for AvxBackend {
    #[inline]
    fn create_full_bitmap(&self, s: &[u8], offset: usize) -> Bitmap {
        let b0 = u8x32::from_slice_unaligned(&s[offset..]);
        let b1 = u8x32::from_slice_unaligned(&s[offset + 32..]);
        Bitmap {
            backslash: cmp2(self.backslash, b0, b1),
            quote: cmp2(self.quote, b0, b1),
            colon: cmp2(self.colon, b0, b1),
            comma: cmp2(self.comma, b0, b1),
            left_brace: cmp2(self.left_brace, b0, b1),
            right_brace: cmp2(self.right_brace, b0, b1),
            left_bracket: cmp2(self.left_bracket, b0, b1),
            right_bracket: cmp2(self.right_bracket, b0, b1),
        }
    }

    #[inline]
    fn create_partial_bitmap(&self, s: &[u8], offset: usize) -> Bitmap {
        match s.len() - offset {
            x if x < 32 => {
                let b0 = u8x32::from_slice_unaligned_partial(&s[offset..]);
                Bitmap {
                    backslash: cmp1(self.backslash, b0),
                    quote: cmp1(self.quote, b0),
                    colon: cmp1(self.colon, b0),
                    comma: cmp1(self.comma, b0),
                    left_brace: cmp1(self.left_brace, b0),
                    right_brace: cmp1(self.right_brace, b0),
                    left_bracket: cmp1(self.left_bracket, b0),
                    right_bracket: cmp1(self.right_bracket, b0),
                }
            }
            32 => {
                let b0 = u8x32::from_slice_unaligned(&s[offset..]);
                Bitmap {
                    backslash: cmp1(self.backslash, b0),
                    quote: cmp1(self.quote, b0),
                    colon: cmp1(self.colon, b0),
                    comma: cmp1(self.comma, b0),
                    left_brace: cmp1(self.left_brace, b0),
                    right_brace: cmp1(self.right_brace, b0),
                    left_bracket: cmp1(self.left_bracket, b0),
                    right_bracket: cmp1(self.right_bracket, b0),
                }
            }
            _ => {
                let b0 = u8x32::from_slice_unaligned(&s[offset..]);
                let b1 = u8x32::from_slice_unaligned_partial(&s[offset + 32..]);
                Bitmap {
                    backslash: cmp2(self.backslash, b0, b1),
                    quote: cmp2(self.quote, b0, b1),
                    colon: cmp2(self.colon, b0, b1),
                    comma: cmp2(self.comma, b0, b1),
                    left_brace: cmp2(self.left_brace, b0, b1),
                    right_brace: cmp2(self.right_brace, b0, b1),
                    left_bracket: cmp2(self.left_bracket, b0, b1),
                    right_bracket: cmp2(self.right_bracket, b0, b1),
                }
            }
        }
    }
}

trait U8x32Ext {
    fn from_slice_unaligned_partial(s: &[u8]) -> Self;
}

impl U8x32Ext for u8x32 {
    #[inline]
    fn from_slice_unaligned_partial(s: &[u8]) -> u8x32 {
        let mut remains = [0u8; 32];
        remains[0..s.len()].copy_from_slice(s);
        u8x32::from_slice_unaligned(&remains[..])
    }
}

#[inline]
fn cmp1(b: u8x32, b0: u8x32) -> u64 {
    b.eq(b0).bitmask() as u64
}

#[inline]
fn cmp2(b: u8x32, b0: u8x32, b1: u8x32) -> u64 {
    cmp1(b, b0) | (b.eq(b1).bitmask() as u64) << 32
}
