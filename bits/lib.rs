//! `bits`

mod bit_all;
mod bit_any;
mod bit_count;
mod bit_excess;
mod bit_get;
mod bit_len;
mod bit_put;
mod bit_rank;
mod bit_select;
pub mod ops {
    pub use crate::bit_all::BitAll;
    pub use crate::bit_any::BitAny;
    pub use crate::bit_count::BitCount;
    pub use crate::bit_excess::BitExcess;
    pub use crate::bit_get::BitGet;
    pub use crate::bit_len::BitLen;
    pub use crate::bit_put::BitPut;
    pub use crate::bit_rank::BitRank;
    pub use crate::bit_select::BitSelect;
}

mod bools;
mod impls;
mod slice;
mod word;

use core::ops::RangeBounds;
pub use word::Word;

pub trait Bits:
    Clone
    + ops::BitLen
    + ops::BitCount
    + ops::BitAll
    + ops::BitAny
    + ops::BitRank
    + ops::BitSelect
    + ops::BitGet
    + ops::BitPut
{
    const BITS: usize;

    #[doc(hidden)]
    const SIZE: usize = Self::BITS / 8;

    fn null() -> Self;
}

#[inline]
fn address<T: Bits>(i: usize) -> (usize, usize) {
    use core::ops::{Div, Rem};
    fn divrem<T, U>(t: T, u: U) -> (<T as Div<U>>::Output, <T as Rem<U>>::Output)
    where
        T: Copy + Div<U> + Rem<U>,
        U: Copy,
    {
        (t / u, t % u)
    }

    divrem(i, T::BITS)
}

/// A utility to clamp the given range into a valid one.
/// Panics if debug is enabled and `min <= i && i <= j && j <= max`.
fn to_range<R: core::ops::RangeBounds<usize>>(r: &R, min: usize, max: usize) -> (usize, usize) {
    use core::ops::Bound::*;

    let (i, j) = (
        match r.start_bound() {
            Included(&s) => s,
            Excluded(&s) => s + 1,
            Unbounded => min,
        },
        match r.end_bound() {
            Included(&e) => e + 1,
            Excluded(&e) => e,
            Unbounded => max,
        },
    );

    debug_assert!(min <= i && i <= j && j <= max);
    (i, j)
}

/// Calculates the minimum number of blocks to store `n` bits.
const fn blocks(n: usize, b: usize) -> usize {
    n / b + (n % b > 0) as usize
}

/// Returns an empty `Vec<T>` with the at least specified capacity in bits.
///
/// ```
/// let v = bits::with_capacity::<u8>(80);
/// // v has no bits, but an enough capacity to store 80 bits.
/// assert_eq!(bits::len(&v), 0);
/// assert_eq!(v.capacity(), 10);
/// ```
pub fn with_capacity<T: Bits>(n: usize) -> Vec<T> {
    let size = blocks(n, T::BITS);
    Vec::with_capacity(size)
}

// pub fn null<T: Block>(n: usize) -> Vec<T> {
//     use core::iter::from_fn;
//     let size = blocks(n, T::BITS);
//     from_fn(|| Some(T::empty())).take(size).collect()
// }

/// The number of binary digits.
///
/// # Examples
///
/// ```
/// let v: &[u8] = &[0, 0, 0];
/// let w: &[u8] = &[];
/// assert_eq!(bits::len(v), 24);
/// assert_eq!(bits::len(w), 0);
/// ```
#[inline]
pub fn len<T>(bits: &T) -> usize
where
    T: ?Sized + bit_len::BitLen,
{
    bit_len::BitLen::len(bits)
}

/// Returns true iif `bits::len(this) == 0`.
///
/// # Examples
///
/// ```
/// let v: &[u64] = &[0, 0, 0];
/// let w: &[u64] = &[];
/// assert!(!bits::is_empty(v));
/// assert!( bits::is_empty(w));
/// ```
#[inline]
pub fn is_empty<T>(bits: &T) -> bool
where
    T: ?Sized + bit_len::BitLen,
{
    bit_len::BitLen::is_empty(bits)
}

#[inline]
pub fn get<T>(bits: &T, i: usize) -> Option<bool>
where
    T: ?Sized + bit_get::BitGet,
{
    bit_get::BitGet::get(bits, i)
}

#[inline]
pub fn test<T>(bits: &T, i: usize) -> bool
where
    T: ?Sized + bit_get::BitGet,
{
    bit_get::BitGet::test(bits, i)
}

#[inline]
pub fn put_1<T>(bits: &mut T, i: usize)
where
    T: ?Sized + bit_put::BitPut,
{
    bit_put::BitPut::put_1(bits, i)
}

#[inline]
pub fn put_0<T>(bits: &mut T, i: usize)
where
    T: ?Sized + bit_put::BitPut,
{
    bit_put::BitPut::put_0(bits, i)
}

/// Counts the occurrences of `1`.
///
/// # Examples
///
/// ```
/// let a: &[u64] = &[];
/// let b: &[u64] = &[0, 0, 0];
/// let c: &[u64] = &[0, 1, 3];
/// assert_eq!(bits::count_1(a), 0);
/// assert_eq!(bits::count_1(b), 0);
/// assert_eq!(bits::count_1(c), 3);
/// ```
#[inline]
pub fn count_1<T>(bits: &T) -> usize
where
    T: ?Sized + bit_count::BitCount,
{
    bit_count::BitCount::count_1(bits)
}

/// Counts the occurrences of `0`.
///
/// # Examples
///
/// ```
/// let a: &[u64] = &[];
/// let b: &[u64] = &[0, 0, 0];
/// let c: &[u64] = &[0, 1, 3];
/// assert_eq!(bits::count_0(a), 0);
/// assert_eq!(bits::count_0(b), 192);
/// assert_eq!(bits::count_0(c), 189);
/// ```
#[inline]
pub fn count_0<T>(bits: &T) -> usize
where
    T: ?Sized + bit_count::BitCount,
{
    bit_count::BitCount::count_0(bits)
}

/// Returns true if all bits are enabled. An empty bits should return true.
///
/// # Examples
///
/// ```
/// let a: &[u64] = &[0, 0, 0];
/// let b: &[u64] = &[];
/// let c: &[u64] = &[!0, !0, !0];
/// assert!(!bits::all(a));
/// assert!( bits::all(b));
/// assert!( bits::all(c));
/// ```
#[inline]
pub fn all<T>(bits: &T) -> bool
where
    T: ?Sized + bit_all::BitAll,
{
    bit_all::BitAll::all(bits)
}

/// Returns true if any bits are enabled. An empty bits should return false.
///
/// # Examples
///
/// ```
/// let b1: &[u64] = &[];
/// let b2: &[u64] = &[0, 0, 0];
/// let b3: &[u64] = &[!0, !0, !0];
/// let b4: &[u64] = &[0, 0, 1];
/// assert!(!bits::any(b1));
/// assert!(!bits::any(b2));
/// assert!( bits::any(b3));
/// assert!( bits::any(b4));
/// ```
#[inline]
pub fn any<T>(bits: &T) -> bool
where
    T: ?Sized + bit_any::BitAny,
{
    bit_any::BitAny::any(bits)
}

#[inline]
pub fn rank_1<T, Index>(bits: &T, index: Index) -> usize
where
    T: ?Sized + bit_rank::BitRank,
    Index: RangeBounds<usize>,
{
    bit_rank::BitRank::rank_1(bits, index)
}

#[inline]
pub fn rank_0<T, Index>(bits: &T, index: Index) -> usize
where
    T: ?Sized + bit_rank::BitRank,
    Index: RangeBounds<usize>,
{
    bit_rank::BitRank::rank_0(bits, index)
}

#[inline]
pub fn excess_1<T, Index>(bits: &T, index: Index) -> usize
where
    T: ?Sized + bit_excess::BitExcess,
    Index: RangeBounds<usize>,
{
    bit_excess::BitExcess::excess_1(bits, index)
}

#[inline]
pub fn excess_0<T, Index>(bits: &T, index: Index) -> usize
where
    T: ?Sized + bit_excess::BitExcess,
    Index: RangeBounds<usize>,
{
    bit_excess::BitExcess::excess_0(bits, index)
}

#[inline]
pub fn select_1<T>(bits: &T, n: usize) -> Option<usize>
where
    T: ?Sized + bit_select::BitSelect,
{
    bit_select::BitSelect::select_1(bits, n)
}

#[inline]
pub fn select_0<T>(bits: &T, n: usize) -> Option<usize>
where
    T: ?Sized + bit_select::BitSelect,
{
    bit_select::BitSelect::select_0(bits, n)
}

#[doc(hidden)]
#[inline]
pub fn word<T, U>(bits: &T, i: usize, n: usize) -> U
where
    T: ?Sized + bit_get::BitGet,
    U: Word,
{
    bit_get::BitGet::word(bits, i, n)
}

#[doc(hidden)]
#[inline]
pub fn put_n<T, N: Word>(bits: &mut T, i: usize, n: usize, mask: N)
where
    T: ?Sized + bit_put::BitPut,
{
    bit_put::BitPut::put_n(bits, i, n, mask)
}
