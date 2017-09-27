use std::cell::Ref;
use bit;
use errors::{ErrorKind, Result, ResultExt};

use super::builder::Inner;
use super::backend::Bitmap;


/// Structural index of a slice of bytes
#[derive(Debug)]
pub struct StructuralIndex<'a> {
    pub(crate) inner: Ref<'a, Inner>,
}

impl<'a> StructuralIndex<'a> {
    /// Calculate the position of colons at `level`, between from `begin` to `end`
    pub fn colon_positions(&self, begin: usize, end: usize, level: usize) -> Option<Vec<usize>> {
        if level < self.inner.b_colon.len() {
            Some(generate_positions(&self.inner.b_colon[level], begin, end))
        } else {
            None
        }
    }

    /// Calculate the position of colons at `level`, between from `begin` to `end`
    pub fn comma_positions(&self, begin: usize, end: usize, level: usize) -> Option<Vec<usize>> {
        if level < self.inner.b_comma.len() {
            Some(generate_positions(&self.inner.b_comma[level], begin, end))
        } else {
            None
        }
    }

    #[allow(missing_docs)]
    pub fn find_field<'s>(&self, record: &'s str, begin: usize, end: usize) -> Result<(&'s str, usize)> {
        let (fsi, fei) =
            find_pre_field_indices(&self.inner.bitmaps, begin, end).chain_err(|| "find_pre_field_indices()")?;
        Ok((&record[fsi..fei], fsi))
    }
}


fn generate_positions(bitmap: &[u64], begin: usize, end: usize) -> Vec<usize> {
    let mut cp = Vec::new();

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
