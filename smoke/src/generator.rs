//! Generators of elements
//!
//! Generator<T> is a fully specified way to get you an arbitrary T object
//!
//! The general interface works very similarly to an iterator, but instead
//! it never stop generating data, thus instead of returning an Option<Item>
//! it returns the Item directly, and takes an extra random generator
//! to generate the next element.

use super::rand::{NumPrimitive, R};
use core::marker::PhantomData;
use std::sync::Arc;

/// Generator for an Item
///
/// The interface is very similar to an Iterator, except `next` is `gen`
pub trait Generator {
    /// Type generated by the generator
    type Item;

    /// Generate the next item
    fn gen(&self, r: &mut R) -> Self::Item;

    /// Map the output of a generator through a function
    ///
    /// ```
    /// use smoke::{Generator, generator::num};
    ///
    /// let generator = num::<u32>().map(|n| n + 1);
    /// ```
    fn map<O, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Item) -> O,
    {
        Map { generator: self, f }
    }

    /// Filter the generated items such that only the item
    /// that matches the predicate 'f' are returned.
    ///
    /// Due to being an user controlled callback, it's not
    /// recommended to do heavy filtering with this function
    /// and instead use a generator that generate data
    /// that is closer to the end result. A general
    /// rule of thumb, is that if the callback accept
    /// less than half the generated value, then it should
    /// probably be refined at the source generator.
    ///
    /// ```
    /// use smoke::{Generator, generator::range};
    /// // u32 number between 1 and 1000 that are odd only
    /// let odd_gen = range(1u32..1000).such_that(|n| (n & 0x1) == 1);
    /// ```
    fn such_that<F>(self, f: F) -> SuchThat<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Item) -> bool + Clone,
    {
        SuchThat {
            retry: 1000,
            generator: self,
            f,
        }
    }

    /// Combine two arbitrary generators into one that generate tuple item of both generators,
    /// transforming generator for A and generator for B into one generator of (A,B)
    ///
    /// ```
    /// use smoke::{Generator, generator::{Num, num}};
    ///
    /// let generator_a : Num<u32> = num();
    /// let generator_b : Num<u64> = num();
    ///
    /// let generator = generator_a.and(generator_b);
    /// ```
    fn and<G>(self, other: G) -> And<Self, G>
    where
        Self: Sized,
    {
        And {
            gen_a: self,
            gen_b: other,
        }
    }

    /// This generator or another one.
    ///
    /// It's not recommended to use this combinator to chain more than
    /// one generator, as the generator on the "left" will have
    /// a 1/2 handicapped at each iteration.
    ///
    /// Prefered `choose()` to do a unbiased choice or `frequency()` to
    /// control the distribution between generator.
    fn or<G>(self, other: G) -> Or<Self, G>
    where
        Self: Sized,
        G: Generator<Item = Self::Item>,
    {
        Or {
            gen_a: self,
            gen_b: other,
        }
    }

    /// Box a generator into a monomorphic fixed-sized type, that is easier to handle
    fn into_boxed(self) -> BoxGenerator<Self::Item>
    where
        Self: Sized + 'static,
    {
        BoxGenerator(Box::new(self))
    }
}

/// A generic generator
pub struct BoxGenerator<T>(Box<dyn Generator<Item = T>>);

impl<T> Generator for BoxGenerator<T> {
    type Item = T;
    fn gen(&self, r: &mut R) -> Self::Item {
        self.0.gen(r)
    }
    fn into_boxed(self) -> BoxGenerator<Self::Item> {
        self
    }
}

/// Constant generator, always return the same value
#[derive(Clone)]
pub struct Constant<T>(T);

impl<T: Clone> Generator for Constant<T> {
    type Item = T;
    fn gen(&self, _: &mut R) -> Self::Item {
        self.0.clone()
    }
}

/// Integer number generator for a numeric T (usize, u{8,16,32,64,128}, signed int, ..)
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

/// Application of a closure on the generated value
#[derive(Clone)]
pub struct Map<G, F> {
    generator: G,
    f: F,
}

impl<O, G: Generator, F> Generator for Map<G, F>
where
    F: Fn(G::Item) -> O + Clone,
{
    type Item = O;
    fn gen(&self, r: &mut R) -> O {
        let x = self.generator.gen(r);
        (self.f)(x)
    }
}

/// Dependent generator where the second items depends on what has been generated by the first generator
pub struct Depends<G, F> {
    src_gen: G,
    dst_gen: F,
}

impl<G1, G2, F> Generator for Depends<G1, F>
where
    G1: Generator,
    G2: Generator,
    F: Fn(&G1::Item) -> G2,
{
    type Item = (G1::Item, G2::Item);
    fn gen(&self, r: &mut R) -> Self::Item {
        let x = self.src_gen.gen(&mut r.sub());
        let g2 = (self.dst_gen)(&x);
        let y = g2.gen(&mut r.sub());
        (x, y)
    }
}

/// Product of 2 generators : G1 x G2
#[derive(Clone)]
pub struct Product2<G1, G2, F> {
    gen1: G1,
    gen2: G2,
    f: F,
}

impl<G1, G2, F> Product2<G1, G2, F> {
    fn new(gen1: G1, gen2: G2, f: F) -> Self {
        Product2 { gen1, gen2, f }
    }
}

impl<O, G1: Generator, G2: Generator, F> Generator for Product2<G1, G2, F>
where
    F: Fn(G1::Item, G2::Item) -> O + Clone,
{
    type Item = O;
    fn gen(&self, r: &mut R) -> Self::Item {
        let x1 = self.gen1.gen(&mut r.sub());
        let x2 = self.gen2.gen(&mut r.sub());
        (self.f)(x1, x2)
    }
}

/// Product of 3 generators : G1 x G2 x G3
#[derive(Clone)]
pub struct Product3<G1, G2, G3, F> {
    gen1: G1,
    gen2: G2,
    gen3: G3,
    f: F,
}

impl<G1, G2, G3, F> Product3<G1, G2, G3, F> {
    fn new(gen1: G1, gen2: G2, gen3: G3, f: F) -> Self {
        Product3 {
            gen1,
            gen2,
            gen3,
            f,
        }
    }
}

impl<O, G1: Generator, G2: Generator, G3: Generator, F> Generator for Product3<G1, G2, G3, F>
where
    F: Fn(G1::Item, G2::Item, G3::Item) -> O + Clone,
{
    type Item = O;
    fn gen(&self, r: &mut R) -> Self::Item {
        let x1 = self.gen1.gen(&mut r.sub());
        let x2 = self.gen2.gen(&mut r.sub());
        let x3 = self.gen3.gen(&mut r.sub());
        (self.f)(x1, x2, x3)
    }
}

/// Generator filtering mechanisms, such that the resulting generator,
/// generate Item elements where the predicate is valid only.
#[derive(Clone)]
pub struct SuchThat<G, F> {
    retry: u32,
    generator: G,
    f: F,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SuchThatRetryFailure;

impl<G: Generator, F> Generator for SuchThat<G, F>
where
    F: Fn(&G::Item) -> bool + Clone,
{
    type Item = G::Item;
    fn gen(&self, r: &mut R) -> Self::Item {
        let mut retry = self.retry;
        loop {
            let x = self.generator.gen(r);
            if (self.f)(&x) {
                break x;
            }
            if retry == 0 {
                std::panic::panic_any(SuchThatRetryFailure);
            } else {
                retry -= 1;
            }
        }
    }
}

/// One of the element from a list
#[derive(Clone)]
pub struct OneOf<T> {
    data: Box<[T]>,
}

impl<T: Clone> Generator for OneOf<T> {
    type Item = T;
    fn gen(&self, r: &mut R) -> Self::Item {
        let nb = r.num_range(0, self.data.len() - 1);
        self.data[nb].clone()
    }
}

/// Choose one of the generator of T arbitrarily
///
/// This is similar to Frequency but without the weights
#[derive(Clone)]
pub struct Choose<T> {
    generators: Arc<Box<[Box<dyn Generator<Item = T>>]>>,
}

impl<T> Choose<T> {
    fn new(vec: Vec<Box<dyn Generator<Item = T>>>) -> Self {
        assert!(!vec.is_empty());
        Choose {
            generators: Arc::new(vec.into()),
        }
    }
}

impl<T> Generator for Choose<T> {
    type Item = T;
    fn gen(&self, r: &mut R) -> Self::Item {
        let nb = r.num_range(0, self.generators.len() - 1);
        (self.generators[nb]).gen(&mut r.sub())
    }
}

/// A weighted random distribution of multiple generators
#[derive(Clone)]
pub struct Frequency<T> {
    frequencies: Box<[usize]>,
    generators: Arc<Box<[WeightedBoxGenerator<T>]>>,
}

/// A Generic Boxed Generator with an associated weight (for frequency)
type WeightedBoxGenerator<T> = (usize, BoxGenerator<T>);

impl<T> Frequency<T> {
    fn new(gens: Vec<(usize, BoxGenerator<T>)>) -> Self {
        let total: usize = gens.iter().map(|x| x.0).sum();
        let mut frequencies = Vec::with_capacity(total);
        for (i, (nb, _)) in gens.iter().enumerate() {
            // push nb times this generator index
            for _ in 0..*nb {
                frequencies.push(i)
            }
        }
        Frequency {
            frequencies: frequencies.into(),
            generators: Arc::new(gens.into()),
        }
    }
}

impl<T> Generator for Frequency<T> {
    type Item = T;
    fn gen(&self, r: &mut R) -> Self::Item {
        let nb = r.num_range(0, self.frequencies.len() - 1);
        let idx = self.frequencies[nb];
        (&self.generators[idx].1).gen(&mut r.sub())
    }
}

/// A product generator of one and another
#[derive(Clone)]
pub struct And<A, B> {
    gen_a: A,
    gen_b: B,
}

impl<A, B, T, U> Generator for And<A, B>
where
    A: Generator<Item = T>,
    B: Generator<Item = U>,
{
    type Item = (T, U);
    fn gen(&self, r: &mut R) -> Self::Item {
        let a = self.gen_a.gen(&mut r.sub());
        let b = self.gen_b.gen(&mut r.sub());
        (a, b)
    }
}

/// An alternative generator between one or another
#[derive(Clone)]
pub struct Or<A, B> {
    gen_a: A,
    gen_b: B,
}

impl<A, B, T> Generator for Or<A, B>
where
    A: Generator<Item = T>,
    B: Generator<Item = T>,
{
    type Item = T;
    fn gen(&self, r: &mut R) -> Self::Item {
        if r.bool() {
            self.gen_a.gen(&mut r.sub())
        } else {
            self.gen_b.gen(&mut r.sub())
        }
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

/// The constant generator: always yield the same value
pub fn constant<T: Clone>(t: T) -> Constant<T> {
    Constant(t)
}

/// Generator for a simple numeric primitive over the whole possible range
pub fn num<T: NumPrimitive>() -> Num<T> {
    Num::<T>::default()
}

/// Generator for a simple numeric primitive in a specific range
pub fn range<T: NumPrimitive>(range: std::ops::Range<T>) -> NumRange<T> {
    NumRange::new(range)
}

/// Choose randomly from a list of T elements
pub fn one_of<T: Clone>(slice: &[T]) -> OneOf<T> {
    let copied: Vec<_> = slice.to_vec();
    OneOf {
        data: copied.into_boxed_slice(),
    }
}

/// Create a generator from multiple generators
///
/// If the vector is empty then a runtime error is thrown
pub fn choose<T>(gens: Vec<Box<dyn Generator<Item = T>>>) -> Choose<T> {
    assert!(!gens.is_empty());
    Choose::new(gens)
}

/// Create a generator from multiple generators and their associated weight distribution list
///
/// For example `frequency(vec!([ (3, A), (7, B) ])` will create a generator
/// which is has 30% (3/(3+7)) to generate from the A generator and
/// 70% (7/(3+7)) to generate from the B generator.
///
/// If the vector is empty then a runtime error is thrown
pub fn frequency<T>(gens: Vec<(usize, Box<dyn Generator<Item = T>>)>) -> Frequency<T> {
    assert!(!gens.is_empty());
    let mut frequencies_gen = Vec::new();
    for (freq, gen) in gens.into_iter() {
        frequencies_gen.push((freq, BoxGenerator(gen)))
    }

    Frequency::new(frequencies_gen)
}

/// Product of 2 generators, figuratively: F(G1, G2)
pub fn product2<G1, G2, F>(gen1: G1, gen2: G2, f: F) -> Product2<G1, G2, F> {
    Product2::new(gen1, gen2, f)
}

/// Product of 3 generators, figuratively: F(G1, G2, G3)
///
/// ```
/// use smoke::generator::{product3, range, num};
/// pub struct Point { x: u32, y: u32, z: u32 }
///
/// let pointgen = product3(num::<u32>(), num::<u32>(), range(1u32..3), |x, y, z| Point { x, y, z });
/// ```
pub fn product3<G1, G2, G3, F>(gen1: G1, gen2: G2, gen3: G3, f: F) -> Product3<G1, G2, G3, F> {
    Product3::new(gen1, gen2, gen3, f)
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

pub fn depends<F, G1, G2>(g1: G1, f: F) -> Depends<G1, F>
where
    G1: Generator,
    G2: Generator,
    F: FnOnce(&G1::Item) -> G2,
{
    Depends {
        src_gen: g1,
        dst_gen: f,
    }
}
