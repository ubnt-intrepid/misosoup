use simd::u8x16;
use simd::x86::sse2::Sse2Bool8ix16;
use super::super::{Backend, Bitmap};


#[allow(missing_docs)]
#[derive(Debug)]
pub struct Sse2Backend {
    backslash: u8x16,
    quote: u8x16,
    colon: u8x16,
    left_brace: u8x16,
    right_brace: u8x16,
}

impl Default for Sse2Backend {
    fn default() -> Self {
        Self {
            backslash: u8x16::splat(b'\\'),
            quote: u8x16::splat(b'"'),
            colon: u8x16::splat(b':'),
            left_brace: u8x16::splat(b'{'),
            right_brace: u8x16::splat(b'}'),
        }
    }
}

impl Backend for Sse2Backend {
    fn create_full_bitmap(&self, s: &[u8], offset: usize) -> Bitmap {
        let b0 = u8x16::load(s, offset);
        let b1 = u8x16::load(s, offset + 16);
        let b2 = u8x16::load(s, offset + 32);
        let b3 = u8x16::load(s, offset + 48);
        Bitmap {
            backslash: cmp4(self.backslash, b0, b1, b2, b3),
            quote: cmp4(self.quote, b0, b1, b2, b3),
            colon: cmp4(self.colon, b0, b1, b2, b3),
            left_brace: cmp4(self.left_brace, b0, b1, b2, b3),
            right_brace: cmp4(self.right_brace, b0, b1, b2, b3),
        }
    }

    fn create_partial_bitmap(&self, s: &[u8], offset: usize) -> Bitmap {
        match s.len() - offset {
            x if x < 16 => {
                let b0 = u8x16::load_partial(s, offset);
                Bitmap {
                    backslash: cmp1(self.backslash, b0),
                    quote: cmp1(self.quote, b0),
                    colon: cmp1(self.colon, b0),
                    left_brace: cmp1(self.left_brace, b0),
                    right_brace: cmp1(self.right_brace, b0),
                }
            }
            16 => {
                let b0 = u8x16::load(s, offset);
                Bitmap {
                    backslash: cmp1(self.backslash, b0),
                    quote: cmp1(self.quote, b0),
                    colon: cmp1(self.colon, b0),
                    left_brace: cmp1(self.left_brace, b0),
                    right_brace: cmp1(self.right_brace, b0),
                }
            }
            x if x < 32 => {
                let b0 = u8x16::load(s, offset);
                let b1 = u8x16::load_partial(s, offset + 16);
                Bitmap {
                    backslash: cmp2(self.backslash, b0, b1),
                    quote: cmp2(self.quote, b0, b1),
                    colon: cmp2(self.colon, b0, b1),
                    left_brace: cmp2(self.left_brace, b0, b1),
                    right_brace: cmp2(self.right_brace, b0, b1),
                }
            }
            32 => {
                let b0 = u8x16::load(s, offset);
                let b1 = u8x16::load(s, offset + 16);
                Bitmap {
                    backslash: cmp2(self.backslash, b0, b1),
                    quote: cmp2(self.quote, b0, b1),
                    colon: cmp2(self.colon, b0, b1),
                    left_brace: cmp2(self.left_brace, b0, b1),
                    right_brace: cmp2(self.right_brace, b0, b1),
                }
            }
            x if x < 48 => {
                let b0 = u8x16::load(s, offset);
                let b1 = u8x16::load(s, offset + 16);
                let b2 = u8x16::load_partial(s, offset + 32);
                Bitmap {
                    backslash: cmp3(self.backslash, b0, b1, b2),
                    quote: cmp3(self.quote, b0, b1, b2),
                    colon: cmp3(self.colon, b0, b1, b2),
                    left_brace: cmp3(self.left_brace, b0, b1, b2),
                    right_brace: cmp3(self.right_brace, b0, b1, b2),
                }
            }
            48 => {
                let b0 = u8x16::load(s, offset);
                let b1 = u8x16::load(s, offset + 16);
                let b2 = u8x16::load(s, offset + 32);
                Bitmap {
                    backslash: cmp3(self.backslash, b0, b1, b2),
                    quote: cmp3(self.quote, b0, b1, b2),
                    colon: cmp3(self.colon, b0, b1, b2),
                    left_brace: cmp3(self.left_brace, b0, b1, b2),
                    right_brace: cmp3(self.right_brace, b0, b1, b2),
                }
            }
            _ => {
                let b0 = u8x16::load(s, offset);
                let b1 = u8x16::load(s, offset + 16);
                let b2 = u8x16::load(s, offset + 32);
                let b3 = u8x16::load_partial(s, offset + 48);
                Bitmap {
                    backslash: cmp4(self.backslash, b0, b1, b2, b3),
                    quote: cmp4(self.quote, b0, b1, b2, b3),
                    colon: cmp4(self.colon, b0, b1, b2, b3),
                    left_brace: cmp4(self.left_brace, b0, b1, b2, b3),
                    right_brace: cmp4(self.right_brace, b0, b1, b2, b3),
                }
            }
        }
    }
}


trait U8x16Ext {
    fn load_partial(s: &[u8], offset: usize) -> Self;
}

impl U8x16Ext for u8x16 {
    #[inline]
    fn load_partial(s: &[u8], offset: usize) -> u8x16 {
        let mut remains = [0u8; 16];
        remains[0..(s.len() - offset)].copy_from_slice(&s[offset..]);
        u8x16::load(&remains, 0)
    }
}


#[inline]
fn cmp1(b: u8x16, b0: u8x16) -> u64 {
    b.eq(b0).move_mask() as u64
}

#[inline]
fn cmp2(b: u8x16, b0: u8x16, b1: u8x16) -> u64 {
    cmp1(b, b0) | (b.eq(b1).move_mask() as u64) << 16
}

#[inline]
fn cmp3(b: u8x16, b0: u8x16, b1: u8x16, b2: u8x16) -> u64 {
    cmp2(b, b0, b1) | (b.eq(b2).move_mask() as u64) << 32
}

#[inline]
fn cmp4(b: u8x16, b0: u8x16, b1: u8x16, b2: u8x16, b3: u8x16) -> u64 {
    cmp3(b, b0, b1, b2) | (b.eq(b3).move_mask() as u64) << 48
}
