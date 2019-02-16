#![allow(non_snake_case)]

//! Useful bit operators

/// Remove the rightmost 1 in `x`
/// ```ignore
/// assert!( R(0b_1110_1000) == 0b_1110_0000 );
/// ```
#[inline]
pub fn R(x: u64) -> u64 {
    x & x.wrapping_sub(1)
}

/// Remove the leftmost 1 in `x`
/// ```ignore
/// assert!( L(0b_1110_1000) == 0b_0110_1000 );
/// ```
#[inline]
pub fn L(x: u64) -> u64 {
    x & !(1u64.wrapping_shl(63 - x.leading_zeros()))
}

/// Extract the rightmost 1 in `x`
/// ```ignore
/// assert!( E(0b_1110_1000) == 0b_0000_1000 );
/// ```
#[inline]
pub fn E(x: u64) -> u64 {
    x & x.wrapping_neg()
}

/// Extract the rightmost 1 in `x` and smear it to the right
/// ```ignore
/// assert!( E(0b_1110_1000) == 0b_0000_1111 );
/// ```
#[inline]
pub fn S(x: u64) -> u64 {
    x ^ x.saturating_sub(1)
}

/// Return the number of leading ones in the binary representation of `x`,
/// starting `pos`.
/// ```ignore
/// assert!( leading_ones(0b_0011_1000_u64, 6) == 3 );
/// ```
#[inline]
pub fn leading_ones(x: u64, pos: u32) -> u32 {
    (!(x << (64 - pos))).leading_zeros()
}
