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
