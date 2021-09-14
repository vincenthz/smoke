use super::super::rand::{NumPrimitive, R};
/// Integer number generator for a numeric T (usize, u{8,16,32,64,128}, signed int, ..)
use super::base::Generator;
use core::marker::PhantomData;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn num_range<T>(start: T, end: T)
    where T: NumPrimitive + PartialOrd
    {
        let (_, mut r) = R::new();

        let range = std::ops::Range::<T>{ start, end };
        let num_range = super::range::<T>(range.clone());

        for _ in 0..1024 {
            let v = num_range.gen(&mut r);
            assert!(range.contains(&v));
        }
    }

    #[test]
    fn i8_num_range() {
        num_range::<i8>(1, 16);
    }

    #[test]
    fn u8_num_range() {
        num_range::<u8>(1, 16);
    }

    #[test]
    fn i16_num_range() {
        num_range::<i16>(1, 16);
    }

    #[test]
    fn u16_num_range() {
        num_range::<u16>(1, 16);
    }

    #[test]
    fn i32_num_range() {
        num_range::<i32>(1, 16);
    }

    #[test]
    fn u32_num_range() {
        num_range::<u32>(1, 16);
    }

    #[test]
    fn i64_num_range() {
        num_range::<i64>(1, 16);
    }

    #[test]
    fn u64_num_range() {
        num_range::<u64>(1, 16);
    }

    #[test]
    fn i128_num_range() {
        num_range::<i128>(1, 16);
    }

    #[test]
    fn u128_num_range() {
        num_range::<u128>(1, 16);
    }

    #[test]
    fn isize_num_range() {
        num_range::<isize>(1, 16);
    }

    #[test]
    fn usize_num_range() {
        num_range::<usize>(1, 16);
    }
}
