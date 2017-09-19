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

pub fn build_structural_indices<B: Backend + Default>(s: &[u8]) {
    let mut bitmaps = build_structural_character_bitmaps::<B>(s);
    remove_unstructural_quotes(&mut bitmaps);

    // remove unstructural colons, left/right braces from bitmap
    let b_string = build_string_mask_bitmap(&bitmaps);
    for (s, b) in izip!(b_string, &mut bitmaps) {
        b.colon ^= s;
        b.left_brace ^= s;
        b.right_brace ^= s;
    }

    let level = 10;
    let _b_level = build_leveled_colon_bitmap(&bitmaps, level);
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


pub fn build_string_mask_bitmap(bitmaps: &[Bitmap]) -> Vec<u64> {
    let mut b_string = Vec::with_capacity(bitmaps.len());

    // The number of quotes in structural quote bitmap
    let mut n = 0;

    for b in bitmaps {
        let mut m_quote = b.quote;
        let mut m_string = 0u64;
        while m_quote != 0 {
            // invert all of bits from the rightmost 1 of `m_quote` to the end
            m_string ^= bit::S(m_quote);
            // remove the rightmost 1 from `m_quote`
            m_quote = bit::R(m_quote);
            n += 1;
        }

        if n.is_odd() {
            m_string ^= !0u64;
        }

        b_string.push(m_string);
    }

    debug_assert!(n.is_even());

    b_string
}


pub fn build_leveled_colon_bitmap(bitmaps: &[Bitmap], level: usize) -> Vec<Vec<u64>> {
    let mut b_level = vec![Vec::with_capacity(bitmaps.len()); level];
    for i in 0..level {
        b_level[i].extend(bitmaps.iter().map(|b| b.colon));
    }

    let mut s = Vec::new();
    for (i, b) in bitmaps.iter().enumerate() {
        let mut m_left = b.left_brace;
        let mut m_right = b.right_brace;

        loop {
            let m_rightbit = bit::E(m_right);
            let mut m_leftbit = bit::E(m_left);
            while m_leftbit != 0 && (m_rightbit == 0 || m_leftbit < m_rightbit) {
                s.push((i, m_leftbit));
                m_left = bit::R(m_left);
                m_leftbit = bit::E(m_left);
            }

            if m_rightbit != 0 {
                let (j, mlb) = s.pop().unwrap();
                m_leftbit = mlb;

                if s.len() > 0 && s.len() <= level {
                    let b = &mut b_level[s.len() - 1];
                    if i == j {
                        b[i] &= !(m_rightbit.wrapping_sub(m_leftbit));
                    } else {
                        b[j] &= m_leftbit.wrapping_sub(1);
                        b[i] &= !(m_rightbit.wrapping_sub(1));
                        for k in j + 1..i {
                            // MEMO: the index is different to the paper:
                            // b_level[s.len()][k]
                            b[k] = 0;
                        }
                    }
                }
            }

            m_right = bit::R(m_right);

            if m_rightbit == 0 {
                break;
            }
        }
    }

    b_level
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
