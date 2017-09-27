use std::cell::RefCell;
use bit;
use num::Integer;
use errors::{Error, ErrorKind, Result, ResultExt};

use super::backend::{Backend, Bitmap};
use super::index::StructuralIndex;


/// A index builder
#[derive(Debug, Default)]
pub struct IndexBuilder<B: Backend> {
    backend: B,
    inner: RefCell<Inner>,
    level: usize,
}

#[derive(Debug, Default)]
pub(crate) struct Inner {
    pub(crate) bitmaps: Vec<Bitmap>,
    pub(crate) b_colon: Vec<Vec<u64>>,
    pub(crate) b_comma: Vec<Vec<u64>>,
}


impl<B: Backend> IndexBuilder<B> {
    #[allow(missing_docs)]
    pub fn new(backend: B, level: usize) -> Self {
        Self {
            backend,
            inner: RefCell::new(Inner {
                bitmaps: vec![],
                b_colon: vec![vec![]; level],
                b_comma: vec![vec![]; level],
            }),
            level,
        }
    }

    /// Build a structural index from a slice of bytes.
    pub fn build(&self, record: &[u8]) -> Result<StructuralIndex> {
        {
            let mut inner = self.inner.borrow_mut();

            trait VecExt<T> {
                fn init(&mut self, len: usize);
            }
            impl<T> VecExt<T> for Vec<T> {
                #[inline]
                fn init(&mut self, len: usize) {
                    self.clear();
                    self.reserve_exact(len);
                }
            }
            let b_len = (record.len() + 63) / 64;
            inner.bitmaps.init(b_len);
            for c in &mut inner.b_colon {
                c.init(b_len);
            }
            for c in &mut inner.b_comma {
                c.init(b_len);
            }

            // Step 1
            build_structural_character_bitmaps(&mut inner.bitmaps, record, &self.backend);

            // Step 2
            remove_unstructural_quotes(&mut inner.bitmaps);

            // Step 3
            remove_unstructural_characters(&mut inner.bitmaps);

            // Step 4
            build_leveled_bitmaps(&mut inner, self.level)?;
        }

        Ok(StructuralIndex {
            inner: self.inner.borrow(),
        })
    }
}



fn build_structural_character_bitmaps<B: Backend>(bitmaps: &mut Vec<Bitmap>, s: &[u8], backend: &B) {
    for i in 0..(s.len() / 64) {
        bitmaps.push(backend.create_full_bitmap(s, i * 64));
    }

    if s.len() % 64 != 0 {
        bitmaps.push(backend.create_partial_bitmap(s, (s.len() / 64) * 64));
    }
}

fn remove_unstructural_quotes(bitmaps: &mut [Bitmap]) {
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

        bitmaps[i].quote &= !(uu >> 63 | u << 1);

        // save the current result for next iteration
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

fn remove_unstructural_characters(bitmaps: &mut [Bitmap]) {
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

        b.colon &= !m_string;
        b.comma &= !m_string;
        b.left_brace &= !m_string;
        b.right_brace &= !m_string;
        b.left_bracket &= !m_string;
        b.right_bracket &= !m_string;
    }

    debug_assert!(n.is_even());
}

fn build_leveled_bitmaps(inner: &mut Inner, level: usize) -> Result<()> {
    for i in 0..level {
        inner.b_colon[i].extend(inner.bitmaps.iter().map(|b| b.colon));
        inner.b_comma[i].extend(inner.bitmaps.iter().map(|b| b.comma));
    }

    let mut s = Vec::new();
    for (i, b) in inner.bitmaps.iter().enumerate() {
        let mut m_left = b.left_brace | b.left_bracket;
        let mut m_right = b.right_brace | b.right_bracket;

        loop {
            let m_rightbit = bit::E(m_right);
            let mut m_leftbit = bit::E(m_left);
            while m_leftbit != 0 && (m_rightbit == 0 || m_leftbit < m_rightbit) {
                let t = m_leftbit & b.left_brace != 0;
                s.push((i, m_leftbit, t));
                m_left = bit::R(m_left);
                m_leftbit = bit::E(m_left);
            }

            if m_rightbit != 0 {
                let (j, mlb, t) = s.pop()
                    .ok_or_else(|| Error::from(ErrorKind::InvalidRecord))
                    .chain_err(|| "s.pop()")?;
                if t != (m_rightbit & b.right_brace != 0) {
                    return Err(Error::from(ErrorKind::InvalidRecord)).chain_err(|| "invalid bracket/brace");
                }
                m_leftbit = mlb;

                if s.len() > 0 && s.len() - 1 < level {
                    let b_colon = &mut inner.b_colon[s.len() - 1];
                    let b_comma = &mut inner.b_comma[s.len() - 1];

                    if i == j {
                        let mask = !m_rightbit.wrapping_sub(m_leftbit);
                        b_colon[i] &= mask;
                        b_comma[i] &= mask;
                    } else {
                        let mask = m_leftbit.wrapping_sub(1);
                        b_colon[j] &= mask;
                        b_comma[j] &= mask;

                        let mask = !m_rightbit.wrapping_sub(1);
                        b_colon[i] &= mask;
                        b_comma[i] &= mask;

                        for k in j + 1..i {
                            b_colon[k] = 0;
                            b_comma[k] = 0;
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

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::IndexBuilder;
    use super::super::backend::{Bitmap, FallbackBackend};

    #[test]
    fn test_structural_character_bitmaps() {
        struct TestCase {
            input: &'static [u8],
            level: usize,
            bitmaps: Vec<Bitmap>,
            b_colon: Vec<Vec<u64>>,
            b_comma: Vec<Vec<u64>>,
        }
        let cases = vec![
            TestCase {
                input: b"{}",
                level: 1,
                bitmaps: vec![
                    Bitmap {
                        backslash: 0,
                        quote: 0,
                        colon: 0,
                        comma: 0,
                        left_brace: 0b0000_0001,
                        right_brace: 0b0000_0010,
                        left_bracket: 0,
                        right_bracket: 0,
                    },
                ],
                b_colon: vec![vec![0]],
                b_comma: vec![vec![0]],
            },
            TestCase {
                input: r#"{"x\"y\\":10}"#.as_bytes(),
                level: 1,
                bitmaps: vec![
                    Bitmap {
                        backslash: 0b_0000_0000_1100_1000,
                        quote: 0b_0000_0001_0000_0010,
                        colon: 0b_0000_0010_0000_0000,
                        comma: 0,
                        left_brace: 0b_0000_0000_0000_0001,
                        right_brace: 0b_0001_0000_0000_0000,
                        left_bracket: 0,
                        right_bracket: 0,
                    },
                ],
                b_colon: vec![vec![0b_0000_0010_0000_0000]],
                b_comma: vec![vec![0b_0000_0000_0000_0000]],
            },
            TestCase {
                input: r#"{ "f1":"a", "f2":{ "e1": true, "e2": "::a" }, "f3":"\"foo\\" }"#.as_bytes(),
                level: 2,
                bitmaps: vec![
                    Bitmap {
                        backslash: 0b_0000_0110_0001_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                        quote: 0b_0000_1000_0000_1010_0100_0010_0010_0100_1000_0000_0100_1000_1001_0010_1010_0100,
                        colon: 0b_0000_0000_0000_0100_0000_0000_0000_1000_0000_0000_1000_0001_0000_0000_0100_0000,
                        comma: 0b_0000_0000_0000_0000_0001_0000_0000_0000_0010_0000_0000_0000_0000_0100_0000_0000,
                        left_brace: 0b_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0010_0000_0000_0000_0001,
                        right_brace: 0b_0010_0000_0000_0000_0000_1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
                        left_bracket: 0,
                        right_bracket: 0,
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
            },
            TestCase {
                input: r#"{ "f1": { "e1": { "d1": true } } }"#.as_bytes(),
                level: 3,
                bitmaps: vec![
                    Bitmap {
                        backslash: 0,
                        quote: 2368548,
                        colon: 4210752,
                        comma: 0,
                        left_brace: 65793,
                        right_brace: 11274289152,
                        left_bracket: 0,
                        right_bracket: 0,
                    },
                ],
                b_colon: vec![vec![64], vec![16448], vec![4210752]],
                b_comma: vec![vec![0], vec![0], vec![0]],
            },
            TestCase {
                input: br#"{ "a": [0, 1, 2] }"#,
                level: 2,
                bitmaps: vec![
                    Bitmap {
                        backslash: 0,
                        quote: 20,
                        colon: 32,
                        comma: 4608,
                        left_brace: 1,
                        right_brace: 131072,
                        left_bracket: 128,
                        right_bracket: 32768,
                    },
                ],
                //    }_ ]2_, 1_,0 [_:" a"_{
                b_colon: vec![
                    vec![0b_0000_0000_0000_0010_0000],
                    vec![0b_0000_0000_0000_0010_0000],
                ],
                b_comma: vec![
                    vec![0b_0000_0000_0000_0000_0000],
                    vec![0b_0000_0001_0010_0000_0000],
                ],
            },
        ];

        for t in cases {
            let index_builder = IndexBuilder::<FallbackBackend>::new(Default::default(), t.level);
            let actual = index_builder.build(t.input).unwrap();
            assert_eq!(t.bitmaps, actual.inner.bitmaps);
            assert_eq!(t.b_colon, actual.inner.b_colon);
            assert_eq!(t.b_comma, actual.inner.b_comma);
        }
    }
}
