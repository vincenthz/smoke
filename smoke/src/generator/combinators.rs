//! combinators

use super::super::rand::R;
use super::base::{BoxGenerator, Generator};
use std::sync::Arc;

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
