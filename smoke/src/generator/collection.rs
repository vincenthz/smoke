use super::super::rand::R;
use super::base::Generator;
use std::mem::MaybeUninit;
use std::ptr;

/// A generator of array of constant length N where elements are defined by a generator
pub struct Array<G, const N: usize> {
    gen: G,
}

impl<G: Generator, const N: usize> Array<G, N> {
    pub fn new(g: G) -> Self {
        Self { gen: g }
    }
}

impl<T, G, const N: usize> Generator for Array<G, N>
where
    G: Generator<Item = T>,
{
    type Item = [T; N];
    fn gen<'a>(&self, r: &mut R) -> Self::Item {
        let mut items: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut sub_r = r.sub();
        for elem in &mut items[..] {
            let cell: T = (self.gen).gen(&mut sub_r);
            unsafe { ptr::write(elem.as_mut_ptr(), cell) }
        }

        // https://github.com/rust-lang/rust/issues/61956
        let ptr = &mut items as *mut _ as *mut [T; N];
        let res = unsafe { ptr.read() };
        core::mem::forget(items);
        res
    }
}

/// A generator of vector of T
#[derive(Clone)]
pub struct Vector<SZ, G> {
    size: SZ,
    t: G,
}

impl<T, SZ, G> Generator for Vector<SZ, G>
where
    SZ: Generator<Item = usize>,
    G: Generator<Item = T>,
{
    type Item = Vec<T>;
    fn gen(&self, r: &mut R) -> Self::Item {
        let sz = (self.size).gen(r);
        let mut v = Vec::with_capacity(sz);
        let mut sub_r = r.sub();
        for _ in 0..sz {
            let cell = self.t.gen(&mut sub_r);
            v.push(cell)
        }
        v
    }
}

/// Create an array of elements where the size is defined of this array is determined by constant generic
/// and the type of elements by the generator
///
/// ```
/// use smoke::generator::{array, range};
/// let array_gen = array::<_,_,32>(range(1u32..45));
/// ```
pub fn array<EL, T, const SZ: usize>(elements: EL) -> Array<EL, SZ>
where
    EL: Generator<Item = T>,
{
    Array { gen: elements }
}

/// Create a vector of elements where the size of the vector is determined by the first generator
/// and the type of elements in the second
pub fn vector<SZ, EL, T>(size: SZ, elements: EL) -> Vector<SZ, EL>
where
    SZ: Generator<Item = usize>,
    EL: Generator<Item = T>,
{
    Vector { size, t: elements }
}
