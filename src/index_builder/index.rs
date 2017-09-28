use std::cell::Ref;
use bit;
use errors::{ErrorKind, Result};
use value::EscapedStr;
use super::builder::Inner;


/// Structural index of a slice of bytes
#[derive(Debug)]
pub struct StructuralIndex<'a, 's> {
    pub(super) record: &'s str,
    pub(super) inner: Ref<'a, Inner>,
}

impl<'a, 's> StructuralIndex<'a, 's> {
    /// Calculate the position of colons at `level`, between from `begin` to `end`
    pub fn colon_positions(&self, begin: usize, end: usize, level: usize, cp: &mut Vec<usize>) -> bool {
        cp.clear();
        if level < self.inner.b_colon.len() {
            generate_positions(&self.inner.b_colon[level], begin, end, cp);
            true
        } else {
            false
        }
    }

    /// Calculate the position of colons at `level`, between from `begin` to `end`
    pub fn comma_positions(&self, begin: usize, end: usize, level: usize, cp: &mut Vec<usize>) -> bool {
        cp.clear();
        if level < self.inner.b_comma.len() {
            generate_positions(&self.inner.b_comma[level], begin, end, cp);
            true
        } else {
            false
        }
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn find_object_field(&self, begin: usize, end: usize) -> Result<(EscapedStr<'s>, usize)> {
        let mut ei = None;

        for i in (begin / 64..(end + 1 + 63) / 64).rev() {
            let mut m_quote = self.inner.bitmaps[i].quote;
            while m_quote != 0 {
                let offset = (i + 1) * 64 - (m_quote.leading_zeros() as usize) - 1;
                if offset < end {
                    if let Some(ei) = ei {
                        let si = offset + 1;
                        return Ok((EscapedStr::from(&self.record[si..ei]), si));
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
    #[inline]
    pub fn find_object_value(&self, begin: usize, end: usize, is_last_field: bool) -> (usize, usize) {
        find_object_value(
            self.record.as_bytes(),
            begin,
            end,
            if is_last_field { b'}' } else { b',' },
        )
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn find_array_value(&self, begin: usize, end: usize) -> (usize, usize) {
        find_array_value(self.record.as_bytes(), begin, end)
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn substr(&self, begin: usize, end: usize) -> &'s str {
        debug_assert!(begin <= end);
        &self.record[begin..end]
    }
}


#[inline]
fn generate_positions(bitmap: &[u64], begin: usize, end: usize, cp: &mut Vec<usize>) {
    for i in begin / 64..(end - 1 + 63) / 64 {
        let mut m_bits = bitmap[i];
        while m_bits != 0 {
            let m_bit = bit::E(m_bits);
            let offset = i * 64 + (m_bit.trailing_zeros() as usize);
            if begin <= offset && offset < end {
                cp.push(offset);
            }
            m_bits = bit::R(m_bits);
        }
    }
}

#[inline]
fn find_object_value(s: &[u8], mut begin: usize, mut end: usize, delim: u8) -> (usize, usize) {
    while begin < end {
        match s[begin] {
            b' ' | b'\t' | b'\r' | b'\n' => begin += 1,
            _ => break,
        }
    }

    let mut seen_delim = false;
    while end > begin {
        match s[end - 1] {
            b' ' | b'\t' | b'\r' | b'\n' => end -= 1,
            s if s == delim && !seen_delim => {
                seen_delim = true;
                end -= 1;
            }
            _ => break,
        }
    }

    (begin, end)
}

#[inline]
fn find_array_value(s: &[u8], mut begin: usize, mut end: usize) -> (usize, usize) {
    while begin < end {
        match s[begin] {
            b' ' | b'\t' | b'\r' | b'\n' => begin += 1,
            _ => break,
        }
    }

    while end >= begin {
        match s[end - 1] {
            b' ' | b'\t' | b'\r' | b'\n' => end -= 1,
            _ => break,
        }
    }

    (begin, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_object_value() {
        struct TestCase {
            input: &'static [u8],
            begin: usize,
            end: usize,
            delim: u8,
            expect: (usize, usize),
        }
        let tests = &[
            TestCase {
                input: br#"{ "a": {}, "b": [] } "#,
                begin: 6,
                end: 10,
                delim: b',',
                expect: (7, 9),
            },
            TestCase {
                input: br#"{ "a": {}, "b": [] } "#,
                begin: 16,
                end: 21,
                delim: b'}',
                expect: (16, 18),
            },
        ];
        for t in tests {
            let actual = find_object_value(t.input, t.begin, t.end, t.delim);
            assert_eq!(actual, t.expect);
        }
    }
}
