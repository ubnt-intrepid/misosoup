use stdsimd::simd::{i8x32, u8x32};
use super::super::{Backend, Bitmap};

#[allow(improper_ctypes)]
extern "C" {
    #[link_name = "llvm.x86.avx2.pmovmskb"]
    fn pmovmskb(a: i8x32) -> i32;
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct AvxBackend {
    backslash: u8x32,
    quote: u8x32,
    colon: u8x32,
    comma: u8x32,
    left_brace: u8x32,
    right_brace: u8x32,
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
        }
    }
}

impl Backend for AvxBackend {
    fn create_full_bitmap(&self, s: &[u8], offset: usize) -> Bitmap {
        let b0 = u8x32::load(s, offset);
        let b1 = u8x32::load(s, offset + 32);
        Bitmap {
            backslash: cmp2(self.backslash, b0, b1),
            quote: cmp2(self.quote, b0, b1),
            colon: cmp2(self.colon, b0, b1),
            comma: cmp2(self.comma, b0, b1),
            left_brace: cmp2(self.left_brace, b0, b1),
            right_brace: cmp2(self.right_brace, b0, b1),
        }
    }

    fn create_partial_bitmap(&self, s: &[u8], offset: usize) -> Bitmap {
        match s.len() - offset {
            x if x < 32 => {
                let b0 = u8x32::load_partial(s, offset);
                Bitmap {
                    backslash: cmp1(self.backslash, b0),
                    quote: cmp1(self.quote, b0),
                    colon: cmp1(self.colon, b0),
                    comma: cmp1(self.comma, b0),
                    left_brace: cmp1(self.left_brace, b0),
                    right_brace: cmp1(self.right_brace, b0),
                }
            }
            32 => {
                let b0 = unsafe { u8x32::load_unchecked(s, offset) };
                Bitmap {
                    backslash: cmp1(self.backslash, b0),
                    quote: cmp1(self.quote, b0),
                    colon: cmp1(self.colon, b0),
                    comma: cmp1(self.comma, b0),
                    left_brace: cmp1(self.left_brace, b0),
                    right_brace: cmp1(self.right_brace, b0),
                }
            }
            _ => {
                let b0 = unsafe { u8x32::load_unchecked(s, offset) };
                let b1 = u8x32::load_partial(s, offset + 32);
                Bitmap {
                    backslash: cmp2(self.backslash, b0, b1),
                    quote: cmp2(self.quote, b0, b1),
                    colon: cmp2(self.colon, b0, b1),
                    comma: cmp2(self.comma, b0, b1),
                    left_brace: cmp2(self.left_brace, b0, b1),
                    right_brace: cmp2(self.right_brace, b0, b1),
                }
            }
        }
    }
}


trait U8x32Ext {
    fn load_partial(s: &[u8], offset: usize) -> Self;
}

impl U8x32Ext for u8x32 {
    #[inline]
    fn load_partial(s: &[u8], offset: usize) -> u8x32 {
        let mut remains = [0u8; 32];
        remains[0..(s.len() - offset)].copy_from_slice(&s[offset..]);
        unsafe { u8x32::load_unchecked(&remains, 0) }
    }
}


#[inline]
fn cmp1(b: u8x32, b0: u8x32) -> u64 {
    unsafe { pmovmskb(b.eq(b0)) as u32 as u64 }
}

#[inline]
fn cmp2(b: u8x32, b0: u8x32, b1: u8x32) -> u64 {
    cmp1(b, b0) | (unsafe { pmovmskb(b.eq(b1)) as u32 as u64 }) << 32
}
