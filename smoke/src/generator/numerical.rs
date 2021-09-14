use super::super::rand::{NumPrimitive, NumRangePrimitive, R};
/// Integer number generator for a numeric T (usize, u{8,16,32,64,128}, signed int, ..)
use super::base::Generator;
use core::marker::PhantomData;
use std::ops::RangeBounds;
use num_traits::cast::AsPrimitive;

#[derive(Copy)]
pub struct Num<T>(PhantomData<T>);

impl<T> Clone for Num<T> {
    fn clone(&self) -> Self {
        Num(self.0)
    }
}

impl<T: NumPrimitive> Default for Num<T> {
    fn default() -> Self {
        Num(PhantomData)
    }
}

impl<T: NumPrimitive> Generator for Num<T> {
    type Item = T;
    fn gen(&self, r: &mut R) -> T {
        r.num()
    }
}

/// Range Primitive generator
#[derive(Clone)]
pub struct NumRange<T>(std::ops::Range<T>);

impl<T> NumRange<T> {
    pub fn new(range: std::ops::Range<T>) -> Self {
        NumRange(range)
    }
}

impl<T: NumPrimitive> Generator for NumRange<T> {
    type Item = T;
    fn gen(&self, r: &mut R) -> T {
        r.num_range(self.0.start, self.0.end)
    }
}

/// Generator for a simple numeric primitive over the whole possible range
pub fn num<T: NumPrimitive>() -> Num<T> {
    Num::<T>::default()
}

/// Generator for a simple numeric primitive in a specific range
pub fn range<T: NumPrimitive>(range: std::ops::Range<T>) -> NumRange<T> {
    NumRange::new(range)
}

#[derive(Clone)]
pub struct NumRangeBounds<T, U>(U, PhantomData<T>)
where T: Sized,
      U: RangeBounds<T>;

impl<T, U> NumRangeBounds<T, U>
where T: Sized,
      U: RangeBounds<T>
{
    pub fn new(range: U) -> Self {
        NumRangeBounds(range, PhantomData)
    }
}

impl<T, U> Generator for NumRangeBounds<T, U>
where U: RangeBounds<T>,
      T: NumRangePrimitive,
      T: AsPrimitive<<T as NumRangePrimitive>::UnsignedType>,
      u32: AsPrimitive<<T as NumRangePrimitive>::UnsignedType>
{
    type Item = T;

    fn gen(&self, r: &mut R) -> T {
        r.num_range_bounds(&self.0)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::rand::NumRangePrimitive;

    fn num_range_bounds<T>(range: impl RangeBounds<T> + Clone)
    where T: NumRangePrimitive,
          T: AsPrimitive<<T as NumRangePrimitive>::UnsignedType>,
          u32: AsPrimitive<<T as NumRangePrimitive>::UnsignedType>
    {
        let (_, mut r) = R::new();
        let num_range_bounds = NumRangeBounds::new(range.clone());

        for _ in 0..1024 {
            let v = num_range_bounds.gen(&mut r);
            assert!(range.contains(&v));
        }
    }

    #[test]
    fn i8_range_bounds() {
        num_range_bounds(1..=16i8);
    }

    #[test]
    fn u8_range_bounds() {
        num_range_bounds(std::ops::Range::<u8>{ start: 1, end: 16 });
    }

    #[test]
    fn i16_range_bounds() {
        num_range_bounds(std::ops::RangeFrom::<i16>{ start: 16 });
    }

    #[test]
    fn u16_range_bounds() {
        num_range_bounds(1u16..16);
    }

    #[test]
    fn i32_range_bounds() {
        num_range_bounds(std::ops::RangeTo::<i32>{ end: 16 });
    }

    #[test]
    fn u32_range_bounds() {
        num_range_bounds(1u32..=16);
    }

    #[test]
    fn i64_range_bounds() {
        num_range_bounds(1i64..16);
    }

    #[test]
    fn u64_range_bounds() {
        let (_, mut r) = R::new();
        let num_range_bounds : NumRangeBounds<u64, std::ops::RangeFull> = NumRangeBounds::new(..);

        for _ in 0..1024 {
            let v = num_range_bounds.gen(&mut r);
            assert!(v <= u64::MAX);
            assert!(u64::MIN <= v);
        }
    }

    #[test]
    fn i128_range_bounds() {
        num_range_bounds(1i128..16);
    }

    #[test]
    fn u128_range_bounds() {
        num_range_bounds(std::ops::RangeInclusive::<u128>::new(1, 16));
    }

    #[test]
    fn isize_range_bounds() {
        num_range_bounds(std::ops::RangeToInclusive::<isize>{ end: 16 });
    }

    #[test]
    fn usize_range_bounds() {
        num_range_bounds(1usize..16);
    }
}
