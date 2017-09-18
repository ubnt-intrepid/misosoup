pub mod backend;

use bit;
use num::Integer;


#[allow(missing_docs)]
pub trait Backend {
    fn create_full_bitmap(&self, s: &[u8], offset: usize) -> Bitmap;
    fn create_partial_bitmap(&self, s: &[u8], offset: usize) -> Bitmap;
}


#[allow(missing_docs)]
#[derive(Debug, PartialEq)]
pub struct Bitmap {
    backslash: u64,
    quote: u64,
    colon: u64,
    left_brace: u64,
    right_brace: u64,
}

#[allow(missing_docs)]
pub fn build_structural_character_bitmaps<B: Backend + Default>(s: &[u8]) -> Vec<Bitmap> {
    let backend = B::default();

    let mut result = Vec::with_capacity((s.len() + 63) / 64);

    for i in 0..(s.len() / 64) {
        result.push(backend.create_full_bitmap(s, i * 64));
    }

    if s.len() % 64 != 0 {
        result.push(backend.create_partial_bitmap(s, (s.len() / 64) * 64));
    }

    result
}


pub fn remove_unstructural_quotes(bitmaps: &mut [Bitmap]) {
    debug_assert!(bitmaps.len() > 0);

    let mut uu = 0u64;
    for i in 0..bitmaps.len() {
        // extract the backslash bitmap, whose succeeding element is a quote.
        let q1 = bitmaps[i].quote;
        let q2 = if i + 1 == bitmaps.len() {
            0
        } else {
            bitmaps[i + 1].quote
        };
        let mut bsq = (q1 >> 1 | q2 << 63) & bitmaps[i].backslash;

        // extract the bits for escaping a quote from `bsq`.
        let mut u = 0u64;
        while bsq != 0 {
            // The target backslash bit.
            let target = bit::E(bsq);
            let pos = 64 - target.leading_zeros();
            if consecutive_ones(&bitmaps[0..i + 1], pos).is_odd() {
                u |= target;
            }
            bsq ^= target; // clear the target bit.
        }

        // Remove unstructural quotes from quote bitmap.
        bitmaps[i].quote &= !(uu >> 63 | u << 1);
        uu = u;
    }
}

/// Compute the length of the consecutive ones in the backslash bitmap starting at `pos`
#[inline]
fn consecutive_ones(b: &[Bitmap], pos: u32) -> u32 {
    let mut ones = bit::leading_ones(b[b.len() - 1].backslash, pos);
    if ones < pos {
        return ones;
    }

    for b in b[0..b.len() - 1].iter().rev() {
        let l = bit::leading_ones(b.backslash, 64);
        if l < 64 {
            return ones + l;
        }
        ones += 64;
    }
    ones
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::backend::Sse2Backend;

    #[test]
    fn test_structural_character_bitmaps() {
        struct TestCase {
            input: &'static [u8],
            expected: Vec<Bitmap>,
        }
        let cases = vec![
            TestCase {
                input: b"{}",
                expected: vec![
                    Bitmap {
                        backslash: 0,
                        quote: 0,
                        colon: 0,
                        left_brace: 0b0000_0001,
                        right_brace: 0b0000_0010,
                    },
                ],
            },
            TestCase {
                input: r#"{"x\"y\\":10}"#.as_bytes(),
                expected: vec![
                    Bitmap {
                        backslash: 0b1100_1000,
                        quote: 0b0001_0001_0010,
                        colon: 0b0010_0000_0000,
                        left_brace: 0b0000_0001,
                        right_brace: 0b0001_0000_0000_0000,
                    },
                ],
            },
        ];

        for case in cases {
            let actual = build_structural_character_bitmaps::<Sse2Backend>(case.input);
            assert_eq!(&actual[..], &case.expected[..]);
        }
    }
}
