use super::{Backend, Bitmap};
use std::u64;

#[allow(missing_docs)]
#[derive(Debug)]
pub struct FallbackBackend {
    backslash: m256i,
    quote: m256i,
    colon: m256i,
    comma: m256i,
    left_brace: m256i,
    right_brace: m256i,
    left_bracket: m256i,
    right_bracket: m256i,
}

impl Default for FallbackBackend {
    fn default() -> Self {
        Self {
            backslash: m256i::splat(b'\\'),
            quote: m256i::splat(b'"'),
            colon: m256i::splat(b':'),
            comma: m256i::splat(b','),
            left_brace: m256i::splat(b'{'),
            right_brace: m256i::splat(b'}'),
            left_bracket: m256i::splat(b'['),
            right_bracket: m256i::splat(b']'),
        }
    }
}

impl Backend for FallbackBackend {
    fn create_full_bitmap(&self, s: &[u8], offset: usize) -> Bitmap {
        let b0 = m256i::load(s, offset);
        let b1 = m256i::load(s, offset + 32);
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

    fn create_partial_bitmap(&self, s: &[u8], offset: usize) -> Bitmap {
        match s.len() - offset {
            x if x < 32 => {
                let b0 = m256i::load_partial(s, offset);
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
                let b0 = m256i::load(s, offset);
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
                let b0 = m256i::load(s, offset);
                let b1 = m256i::load_partial(s, offset + 32);
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


#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
struct m256i([u64; 4]);

impl m256i {
    #[inline]
    fn splat(x: u8) -> Self {
        m256i([x as u64 * 0x_0101_0101_0101_0101_u64; 4])
    }

    #[inline]
    fn load(s: &[u8], i: usize) -> m256i {
        debug_assert!(i + 31 < s.len());
        m256i([
            slice_to_u64(s, i),
            slice_to_u64(s, i + 8),
            slice_to_u64(s, i + 16),
            slice_to_u64(s, i + 24),
        ])
    }

    #[inline]
    fn load_partial(s: &[u8], i: usize) -> m256i {
        let mut result = [0u64; 4];
        for x in i..s.len() {
            result[(x - i) / 8] |= (s[x] as u64) << ((7 - ((x - i) & 7)) * 8);
        }
        m256i(result)
    }

    #[inline]
    fn eq(self, other: m256i) -> m256i {
        m256i([
            bytewise_equal(self.0[0], other.0[0]),
            bytewise_equal(self.0[1], other.0[1]),
            bytewise_equal(self.0[2], other.0[2]),
            bytewise_equal(self.0[3], other.0[3]),
        ])
    }

    #[inline]
    fn move_mask(self) -> u64 {
        let f = 0x_8040_2010_0804_0201_u64;
        ((self.0[0].wrapping_mul(f) >> 56) & 0x0000_00FF_u64 | (self.0[1].wrapping_mul(f) >> 48) & 0x0000_FF00_u64
            | (self.0[2].wrapping_mul(f) >> 40) & 0x00FF_0000_u64 | (self.0[3].wrapping_mul(f) >> 32) & 0xFF00_0000_u64)
    }
}


#[inline]
fn cmp1(b: m256i, b0: m256i) -> u64 {
    b.eq(b0).move_mask()
}

#[inline]
fn cmp2(b: m256i, b0: m256i, b1: m256i) -> u64 {
    cmp1(b, b0) | (b.eq(b1).move_mask() << 32)
}

#[inline]
fn slice_to_u64(s: &[u8], offset: usize) -> u64 {
    let mut res = 0u64;
    for (i, x) in s[offset..offset + 8].iter().enumerate() {
        res |= (*x as u64) << ((7 - i) * 8);
    }
    res
}

#[inline]
fn bytewise_equal(mut x: u64, y: u64) -> u64 {
    const LO: u64 = u64::MAX / 0xFF;
    const HI: u64 = LO << 7;
    x ^= y;
    !((((x & !HI) + !HI) | x) >> 7) & LO
}
