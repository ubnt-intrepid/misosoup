//! Definition of index builder and structural indices

pub mod backend;

use bit;
use errors::{ErrorKind, Result};
use num::Integer;
use self::backend::Backend;


/// Structural character bitmaps
#[allow(missing_docs)]
#[derive(Debug, PartialEq)]
pub struct Bitmap {
    pub backslash: u64,
    pub quote: u64,
    pub colon: u64,
    pub comma: u64,
    pub left_brace: u64,
    pub right_brace: u64,
}


/// Structural index of a slice of bytes
#[derive(Debug, PartialEq)]
pub struct StructuralIndex {
    /// Structural character bitmaps
    pub bitmaps: Vec<Bitmap>,
    /// Leveled colon bitmap
    pub b_colon: Vec<Vec<u64>>,
    /// Leveled comma bitmap
    pub b_comma: Vec<Vec<u64>>,
    /// Leveled right-brace bitmap
    pub b_rbrace: Vec<Vec<u64>>,
}

impl StructuralIndex {
    /// Calculate the position of colons at `level`, between from `begin` to `end`
    pub fn colon_positions(&self, begin: usize, end: usize, level: usize) -> Vec<usize> {
        generate_colon_positions(&self.b_colon[level], begin, end)
    }

    #[allow(missing_docs)]
    pub fn find_field<'s>(&self, record: &'s str, begin: usize, end: usize) -> Result<&'s str> {
        let (fsi, fei) = find_pre_field_indices(&self.bitmaps, begin, end)?;
        Ok(&record[fsi..fei])
    }

    #[allow(missing_docs)]
    pub fn find_value<'s>(&self, record: &'s str, begin: usize, end: usize, level: usize) -> Result<&'s str> {
        let (vsi, vei) = find_post_value_indices(&self.b_comma[level], &self.b_rbrace[level], begin, end)?;
        Ok(record[vsi..vei].trim())
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
            b.comma &= !s;
            b.left_brace &= !s;
            b.right_brace &= !s;
        }

        // Step4: build leveled bitmap of colons, from (cleaned) character bitmap
        let (b_colon, b_comma, b_rbrace) = build_leveled_bitmaps(&bitmaps, level);

        StructuralIndex {
            bitmaps,
            b_colon,
            b_comma,
            b_rbrace,
        }
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

fn build_leveled_bitmaps(bitmaps: &[Bitmap], level: usize) -> (Vec<Vec<u64>>, Vec<Vec<u64>>, Vec<Vec<u64>>) {
    let mut b_colon = vec![Vec::with_capacity(bitmaps.len()); level];
    let mut b_comma = vec![Vec::with_capacity(bitmaps.len()); level];
    let mut b_rbrace = vec![Vec::with_capacity(bitmaps.len()); level];
    for i in 0..level {
        b_colon[i].extend(bitmaps.iter().map(|b| b.colon));
        b_comma[i].extend(bitmaps.iter().map(|b| b.comma));
        b_rbrace[i].extend(bitmaps.iter().map(|b| b.right_brace));
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
                    let b_colon = &mut b_colon[s.len() - 1];
                    let b_comma = &mut b_comma[s.len() - 1];
                    let b_rbrace = &mut b_rbrace[s.len() - 1];
                    if i == j {
                        b_colon[i] &= !(m_rightbit.wrapping_sub(m_leftbit));
                        b_comma[i] &= !(m_rightbit.wrapping_sub(m_leftbit));
                        b_rbrace[i] &= !(m_rightbit.wrapping_sub(m_leftbit)) << 1 | 1;
                    } else {
                        b_colon[j] &= m_leftbit.wrapping_sub(1);
                        b_comma[j] &= m_leftbit.wrapping_sub(1);
                        b_rbrace[j] &= m_leftbit.wrapping_sub(1) << 1 | 1;
                        b_colon[i] &= !(m_rightbit.wrapping_sub(1));
                        b_comma[i] &= !(m_rightbit.wrapping_sub(1));
                        b_rbrace[i] &= !(m_rightbit.wrapping_sub(1)) << 1 | 1;
                        for k in j + 1..i {
                            b_colon[k] = 0;
                            b_comma[k] = 0;
                            b_rbrace[k] = 0;
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

    (b_colon, b_comma, b_rbrace)
}

fn generate_colon_positions(b_colon: &[u64], begin: usize, end: usize) -> Vec<usize> {
    let mut cp = Vec::new();

    for i in begin / 64..(end - 1 + 63) / 64 {
        let mut m_colon = b_colon[i];
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

fn find_pre_field_indices(bitmaps: &[Bitmap], begin: usize, end: usize) -> Result<(usize, usize)> {
    let mut ei = None;

    for i in (begin / 64..(end + 1 + 63) / 64).rev() {
        let mut m_quote = bitmaps[i].quote;
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

fn find_post_value_indices(b_comma: &[u64], b_rbrace: &[u64], begin: usize, end: usize) -> Result<(usize, usize)> {
    for i in begin / 64..(end - 1 + 63) / 64 {
        let mut m_delim = b_comma[i] | b_rbrace[i];
        while m_delim != 0 {
            let m_bit = bit::E(m_delim);
            let offset = i * 64 + (m_bit.trailing_zeros() as usize);
            if begin <= offset && offset < end {
                return Ok((begin, offset));
            }
            m_delim = bit::R(m_delim);
        }
    }
    Err(ErrorKind::InvalidRecord.into())
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
                            comma: 0,
                            left_brace: 0b0000_0001,
                            right_brace: 0b0000_0010,
                        },
                    ],
                    b_colon: vec![vec![0]],
                    b_comma: vec![vec![0]],
                    b_rbrace: vec![vec![2]],
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
                            comma: 0,
                            left_brace: 0b_0000_0000_0000_0001,
                            right_brace: 0b_0001_0000_0000_0000,
                        },
                    ],
                    b_colon: vec![vec![0b_0000_0010_0000_0000]],
                    b_comma: vec![vec![0b_0000_0000_0000_0000]],
                    b_rbrace: vec![vec![4096]],
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
                            comma: 0b_0000_0000_0000_0000_0001_0000_0000_0000_0010_0000_0000_0000_0000_0100_0000_0000,
                            left_brace: 0b_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0010_0000_0000_0000_0001,
                            right_brace: 0b_0010_0000_0000_0000_0000_1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                        },
                    ],
                    b_colon: vec![
                        vec![
                            0b_0000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0001_0000_0000_0100_0000,
                        ],
                        vec![
                            0b_0000_0000_0000_0100_0000_0000_0000_1000_0000_0000_1000_0001_0000_0000_0100_0000,
                        ],
                    ],
                    b_comma: vec![
                        vec![
                            0b_0000_0000_0000_0000_0001_0000_0000_0000_0000_0000_0000_0000_0000_0100_0000_0000,
                        ],
                        vec![
                            0b_0000_0000_0000_0000_0001_0000_0000_0000_0010_0000_0000_0000_0000_0100_0000_0000,
                        ],
                    ],
                    b_rbrace: vec![
                        vec![
                            0b_0010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                        ],
                        vec![
                            0b_0010_0000_0000_0000_0000_1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                        ],
                    ],
                },
            },
            TestCase {
                input: r#"{ "f1": { "e1": { "d1": true } } }"#.as_bytes(),
                level: 3,
                expected: StructuralIndex {
                    bitmaps: vec![
                        Bitmap {
                            backslash: 0,
                            quote: 2368548,
                            colon: 4210752,
                            comma: 0,
                            left_brace: 65793,
                            right_brace: 11274289152,
                        },
                    ],
                    b_colon: vec![vec![64], vec![16448], vec![4210752]],
                    b_comma: vec![vec![0], vec![0], vec![0]],
                    b_rbrace: vec![
                        vec![0b_0010_0000_0000_0000_0000_0000_0000_0000_0000],
                        vec![0b_0010_1000_0000_0000_0000_0000_0000_0000_0000],
                        vec![0b_0010_1010_0000_0000_0000_0000_0000_0000_0000],
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
