//! Definition of index builder and structural indices

pub mod backend;

use bit;
use errors::{ErrorKind, Result};
use num::Integer;
use self::backend::Backend;


/// Structural character bitmaps
#[derive(Debug, PartialEq)]
pub struct Bitmap {
    /// backslash (`\`)
    pub backslash: u64,
    /// quote (`"`)
    pub quote: u64,
    /// colon (`:`)
    pub colon: u64,
    /// left brace (`{`)
    pub left_brace: u64,
    /// right brace (`}`)
    pub right_brace: u64,
}


/// Structural index of a slice of bytes
#[derive(Debug, PartialEq)]
pub struct StructuralIndex {
    /// Structural character bitmaps
    pub bitmaps: Vec<Bitmap>,
    /// Leveled colon bitmap
    pub b_level: Vec<Vec<u64>>,
}

impl StructuralIndex {
    /// Calculate the position of colons at `level`, between from `begin` to `end`
    pub fn colon_positions(&self, begin: usize, end: usize, level: usize) -> Vec<usize> {
        let mut cp = Vec::new();
        for i in begin / 64..(end - 1 + 63) / 64 {
            let mut m_colon = self.b_level[level][i];
            while m_colon != 0 {
                let m_bit = bit::E(m_colon);
                let offset = i * 64 + (m_bit.trailing_zeros() as usize);
                if begin <= offset && offset < end {
                    cp.push(offset);
                }
                m_colon = bit::R(m_colon);
            }
        }
        cp
    }

    #[allow(missing_docs)]
    pub fn find_field(&self, begin: usize, end: usize) -> Result<(usize, usize)> {
        let mut ei = None;
        for i in (begin / 64..(end + 1 + 63) / 64).rev() {
            let mut m_quote = self.bitmaps[i].quote;
            while m_quote != 0 {
                let offset = (i + 1) * 64 - (m_quote.leading_zeros() as usize) - 1;
                if offset < end {
                    if let Some(ei) = ei {
                        let si = offset + 1;
                        return Ok((si, ei));
                    } else {
                        ei = Some(offset);
                    }
                }
                m_quote = bit::L(m_quote);
            }
        }

        Err(ErrorKind::InvalidRecord.into())
    }

    #[allow(missing_docs)]
    pub fn find_value(&self, record: &[u8], begin: usize, end: usize, last: bool) -> Result<(usize, usize)> {
        let delim = if last { b'}' } else { b',' };
        let pos = record[begin..end]
            .iter()
            .rposition(|&b| b == delim)
            .ok_or_else(|| ErrorKind::InvalidRecord)?;
        Ok((begin, begin + pos))
    }
}

/// A index builder
#[derive(Debug, Default)]
pub struct IndexBuilder<B: Backend> {
    backend: B,
}

impl<B: Backend> IndexBuilder<B> {
    #[allow(missing_docs)]
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Build a structural index from a slice of bytes.
    pub fn build(&self, record: &[u8], level: usize) -> StructuralIndex {
        // Step1: build character bitmap of structural characters ('\', '"', ':', '{', '}')
        let mut bitmaps = build_structural_character_bitmaps(record, &self.backend);

        // Step2: remove unstrucural quotes
        let b_quote = build_unstructural_quote_bitmap(&bitmaps);
        for (b, q) in izip!(&mut bitmaps, b_quote) {
            b.quote &= !q;
        }

        // Step3: remove unstructural colons, left/right braces from bitmap
        let b_string = build_string_mask_bitmap(&bitmaps);
        for (b, s) in izip!(&mut bitmaps, b_string) {
            b.colon &= !s;
            b.left_brace &= !s;
            b.right_brace &= !s;
        }

        // Step4: build leveled bitmap of colons, from (cleaned) character bitmap
        let b_level = build_leveled_colon_bitmap(&bitmaps, level);

        StructuralIndex { bitmaps, b_level }
    }
}



fn build_structural_character_bitmaps<B: Backend>(s: &[u8], backend: &B) -> Vec<Bitmap> {
    let mut result = Vec::with_capacity((s.len() + 63) / 64);

    for i in 0..(s.len() / 64) {
        result.push(backend.create_full_bitmap(s, i * 64));
    }

    if s.len() % 64 != 0 {
        result.push(backend.create_partial_bitmap(s, (s.len() / 64) * 64));
    }

    result
}

fn build_unstructural_quote_bitmap(bitmaps: &[Bitmap]) -> Vec<u64> {
    debug_assert!(bitmaps.len() > 0);

    let mut b_quote = Vec::with_capacity(bitmaps.len());

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

        b_quote.push(uu >> 63 | u << 1);

        // save the current result for next iteration
        uu = u;
    }

    b_quote
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

fn build_string_mask_bitmap(bitmaps: &[Bitmap]) -> Vec<u64> {
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

fn build_leveled_colon_bitmap(bitmaps: &[Bitmap], level: usize) -> Vec<Vec<u64>> {
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

                if s.len() > 0 && s.len() - 1 < level {
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
            level: usize,
            expected: StructuralIndex,
        }
        let cases = vec![
            TestCase {
                input: b"{}",
                level: 1,
                expected: StructuralIndex {
                    bitmaps: vec![
                        Bitmap {
                            backslash: 0,
                            quote: 0,
                            colon: 0,
                            left_brace: 0b0000_0001,
                            right_brace: 0b0000_0010,
                        },
                    ],
                    b_level: vec![vec![0]],
                },
            },
            TestCase {
                input: r#"{"x\"y\\":10}"#.as_bytes(),
                level: 1,
                expected: StructuralIndex {
                    bitmaps: vec![
                        Bitmap {
                            backslash: 0b_0000_0000_1100_1000,
                            quote: 0b_0000_0001_0000_0010,
                            colon: 0b_0000_0010_0000_0000,
                            left_brace: 0b_0000_0000_0000_0001,
                            right_brace: 0b_0001_0000_0000_0000,
                        },
                    ],
                    b_level: vec![vec![0b_0000_0010_0000_0000]],
                },
            },
            TestCase {
                input: r#"{ "f1":"a", "f2":{ "e1": true, "e2": "::a" }, "f3":"\"foo\\" }"#.as_bytes(),
                level: 2,
                expected: StructuralIndex {
                    bitmaps: vec![
                        Bitmap {
                            backslash: 0b_0000_0110_0001_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                            quote: 0b_0000_1000_0000_1010_0100_0010_0010_0100_1000_0000_0100_1000_1001_0010_1010_0100,
                            colon: 0b_0000_0000_0000_0100_0000_0000_0000_1000_0000_0000_1000_0001_0000_0000_0100_0000,
                            left_brace: 0b_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0010_0000_0000_0000_0001,
                            right_brace: 0b_0010_0000_0000_0000_0000_1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                        },
                    ],
                    b_level: vec![
                        vec![
                            0b_0000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0001_0000_0000_0100_0000,
                        ],
                        vec![
                            0b_0000_0000_0000_0100_0000_0000_0000_1000_0000_0000_1000_0001_0000_0000_0100_0000,
                        ],
                    ],
                },
            },
        ];

        let index_builder = IndexBuilder::<Sse2Backend>::default();
        for t in cases {
            let actual = index_builder.build(t.input, t.level);
            assert_eq!(t.expected, actual);
        }
    }
}
