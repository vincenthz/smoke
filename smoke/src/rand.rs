//! Various random generation for basic types
//!
//! This implement a simple linear shifting based random generator,
//! it only produces pseudo random and shouldn't be used in any other
//! settings than here (where the quality of randomness doesn't really matter too much)
//!
//! This is just a starting point, in a later version:
//! * optimise the random number generator
//! * remove the biases
//! * add some multiple cases f32/f64 generators

use core::num::{
    NonZeroIsize, NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};

/// Seed of random generation
///
/// There should be only one instance of this for a given instance of a test suite,
/// so that in case of issues, the printed seed of the failed attempt can be re-used
/// as test driven development
///
/// All pseudo random generators need to be derived from this
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Seed(u128);

/// A pseudo random generator at a given time
///
/// it can created from seed using `R::from_seed`, or
/// from another pseudo random generator using `.sub()`
/// as to create a hierarchy (or a tree) of generator.
///
pub struct R(u64, u64);

impl Seed {
    /// Create a new random seed, using the system time and the thread-id.
    ///
    /// Whilst this is not particularly random, we just need a little randomization
    /// not a full blown unguessable entropy. The quality of this randomness
    /// is not particularly important or interesting.
    pub fn generate() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::SystemTime;

        let mut hasher = DefaultHasher::new();

        // get the system time and hash it
        let now = SystemTime::now();
        now.hash(&mut hasher);
        let r1 = u128::from(hasher.finish());

        // append some randomized stuff on top
        let tid = std::thread::current().id();
        tid.hash(&mut hasher);
        let r2 = u128::from(hasher.finish());

        let r = (r1 << 64) | r2;
        Seed::from(r)
    }
}

impl From<u128> for Seed {
    fn from(u: u128) -> Seed {
        Seed(u)
    }
}

impl std::str::FromStr for Seed {
    type Err = &'static str;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let chunk = str
            .split('-')
            .map(|e| u32::from_str_radix(e, 16))
            .collect::<Vec<_>>();
        if chunk.len() == 4 {
            match (&chunk[0], &chunk[1], &chunk[2], &chunk[3]) {
                (Ok(a), Ok(b), Ok(c), Ok(d)) => {
                    let seed =
                        (*a as u128) << 96 | (*b as u128) << 64 | (*c as u128) << 32 | (*d as u128);
                    Ok(Seed::from(seed))
                }
                (Err(_), _, _, _) => Err("cannot parse 1st element as hexadecimal integer"),
                (_, Err(_), _, _) => Err("cannot parse 2nd element as hexadecimal integer"),
                (_, _, Err(_), _) => Err("cannot parse 3rd element as hexadecimal integer"),
                (_, _, _, Err(_)) => Err("cannot parse 4th element as hexadecimal integer"),
            }
        } else {
            Err("expecting 4 hexadecimal values separated by -")
        }
    }
}

impl std::fmt::Display for Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a0 = (self.0 >> 96) as u32;
        let a1 = (self.0 >> 64) as u32;
        let a2 = (self.0 >> 32) as u32;
        let a3 = self.0 as u32;
        write!(f, "{:08X}-{:08X}-{:08X}-{:08X}", a0, a1, a2, a3)
    }
}

const MUL_FACTOR: u64 = 636_4136_2238_4679_3005;

impl R {
    pub fn new() -> (Seed, Self) {
        let seed = Seed::generate();
        let r = Self::from_seed(seed);
        (seed, r)
    }

    pub fn sub(&mut self) -> Self {
        let r0 = self.0;
        let r1 = self.1;
        let n = self.next();
        R(r0.wrapping_mul(n as u64), r1.wrapping_add(n as u64))
    }

    pub fn from_seed(seed: Seed) -> Self {
        R((seed.0 >> 64) as u64, seed.0 as u64)
    }

    pub(crate) fn next(&mut self) -> u32 {
        let old_state = self.0;
        self.0 = old_state.wrapping_mul(MUL_FACTOR).wrapping_add(self.1 | 1);
        let xor_shifted = (((old_state >> 18) ^ old_state) >> 27) as u32;
        let rot = (old_state >> 59) as u32;
        xor_shifted.rotate_right(rot)
    }

    pub fn next_bytes(&mut self, buf: &mut [u8]) {
        const SZ_NEXT: usize = 4;
        let chunk = buf.len() / SZ_NEXT;
        let rem = buf.len() % SZ_NEXT;
        for i in 0..chunk {
            let start = i * SZ_NEXT;
            let out = self.next().to_le_bytes();
            buf[start..start + SZ_NEXT].copy_from_slice(&out)
        }

        if rem > 0 {
            let last = self.next().to_le_bytes();
            let start = buf.len() - rem;
            buf[start..].copy_from_slice(&last[0..rem])
        }
    }

    pub fn ascii(&mut self) -> char {
        loop {
            let v = self.next() % 0x80;
            if let Some(c) = std::char::from_u32(v) {
                break c;
            }
        }
    }

    pub fn codepoint(&mut self) -> char {
        loop {
            let v = self.next() % 0x11_0000;
            if let Some(c) = std::char::from_u32(v) {
                break c;
            }
        }
    }

    pub fn bool(&mut self) -> bool {
        (self.next() % 2) == 1
    }

    pub fn num<T: NumPrimitive>(&mut self) -> T {
        T::num(self)
    }

    pub fn num_range<T: NumPrimitive>(&mut self, min_value: T, max_value: T) -> T {
        T::num_range(self, min_value, max_value)
    }

    pub fn array_num<T: NumPrimitive>(&mut self, buf: &mut [T]) {
        for b in buf.iter_mut() {
            *b = T::num(self)
        }
    }

    pub fn array_num_range<T: NumPrimitive>(&mut self, min_value: T, max_value: T, buf: &mut [T]) {
        for b in buf.iter_mut() {
            *b = T::num_range(self, min_value, max_value)
        }
    }
}

/// Various instance of numbers generation for primitive num
/// types (u8, u16, ..., u128, i8, ..., NonZeroU8, ...)
pub trait NumPrimitive: Copy {
    /// Return a new value in the whole possible domain of Self
    fn num(r: &mut R) -> Self;

    /// Return a new value between min_value and max_value (both included)
    fn num_range(r: &mut R, min_value: Self, max_value: Self) -> Self;
}

/*
impl NumPrimitive for char {
    fn num(r: &mut R) -> Self {
    }
    fn range(r: &mut R, min_value: Self, max_value: Self) -> Self {
        assert!(min_value <= max_value);
        loop {
            let m = u32::from(min_value);
            let diff = u32::from(max_value) - m;
            let r = r.next() % diff;
            match (m + r).try_into() {
                Ok(c) => break c,
                Err(_) => {}
            }
        }
    }
}
*/

impl NumPrimitive for u8 {
    fn num(r: &mut R) -> Self {
        r.next() as u8
    }
    fn num_range(r: &mut R, min_value: Self, max_value: Self) -> Self {
        assert!(min_value <= max_value);
        let diff = max_value - min_value + 1;
        min_value + (r.next() as Self % diff)
    }
}

impl NumPrimitive for u16 {
    fn num(r: &mut R) -> Self {
        r.next() as Self
    }

    fn num_range(r: &mut R, min_value: Self, max_value: Self) -> Self {
        assert!(min_value <= max_value);
        let diff = max_value - min_value + 1;
        min_value + (r.next() as Self % diff)
    }
}

impl NumPrimitive for u32 {
    fn num(r: &mut R) -> Self {
        r.next()
    }
    fn num_range(r: &mut R, min_value: Self, max_value: Self) -> Self {
        assert!(min_value <= max_value);
        let diff = max_value - min_value + 1;
        min_value + (u32::num(r) % diff)
    }
}

impl NumPrimitive for u64 {
    fn num(r: &mut R) -> Self {
        let v1 = r.next() as u64;
        let v2 = r.next() as u64;
        v1 << 32 | v2
    }
    fn num_range(r: &mut R, min_value: Self, max_value: Self) -> Self {
        assert!(min_value <= max_value);
        let diff = max_value - min_value + 1;
        if diff > 0xffff_ffff {
            let v = Self::num(r) % diff;
            min_value + v
        } else {
            min_value + (r.next() as Self % diff)
        }
    }
}

impl NumPrimitive for u128 {
    fn num(r: &mut R) -> Self {
        let v1 = r.next() as u128;
        let v2 = r.next() as u128;
        let v3 = r.next() as u128;
        let v4 = r.next() as u128;
        v1 << 96 | v2 << 64 | v3 << 32 | v4
    }
    fn num_range(r: &mut R, min_value: Self, max_value: Self) -> Self {
        assert!(min_value <= max_value);
        let diff = max_value - min_value + 1;
        if diff > 0xffff_ffff {
            let v = Self::num(r) % diff;
            min_value + v
        } else {
            min_value + (r.next() as Self % diff)
        }
    }
}

impl NumPrimitive for usize {
    fn num(r: &mut R) -> Self {
        if std::mem::size_of::<usize>() <= 4 {
            u32::num(r) as usize
        } else if std::mem::size_of::<usize>() == 8 {
            u64::num(r) as usize
        } else {
            u128::num(r) as usize
        }
    }
    fn num_range(r: &mut R, min_value: Self, max_value: Self) -> Self {
        assert!(min_value <= max_value);
        let diff = max_value - min_value + 1;
        if diff > 0xffff_ffff {
            let v = Self::num(r) % diff;
            min_value + v
        } else {
            min_value + (r.next() as Self % diff)
        }
    }
}

// unsigned -> signed cast based implementations

macro_rules! define_NumPrimitive_impl_signed {
    ($signed_ty:ty, $unsigned_ty:ty) => {
        impl NumPrimitive for $signed_ty {
            fn num(r: &mut R) -> Self {
                <$unsigned_ty>::num(r) as $signed_ty
            }
            fn num_range(r: &mut R, min_value: Self, max_value: Self) -> Self {
                assert!(min_value <= max_value);
                <$unsigned_ty>::num_range(r, min_value as $unsigned_ty, max_value as $unsigned_ty)
                    as $signed_ty
            }
        }
    };
}

define_NumPrimitive_impl_signed!(i8, u8);
define_NumPrimitive_impl_signed!(i16, u16);
define_NumPrimitive_impl_signed!(i32, u32);
define_NumPrimitive_impl_signed!(i64, u64);
define_NumPrimitive_impl_signed!(i128, u128);
define_NumPrimitive_impl_signed!(isize, usize);

// retry Ty -> NonZeroTy convertion based implementation

macro_rules! define_NumPrimitive_impl_nonzero {
    ($non_zero_ty:ty, $src_ty:ty) => {
        impl NumPrimitive for $non_zero_ty {
            fn num(r: &mut R) -> Self {
                loop {
                    match <$non_zero_ty>::new(<$src_ty>::num(r)) {
                        None => {}
                        Some(v) => break v,
                    }
                }
            }
            fn num_range(r: &mut R, min_value: Self, max_value: Self) -> Self {
                assert!(min_value <= max_value);
                loop {
                    match <$non_zero_ty>::new(<$src_ty>::num_range(
                        r,
                        min_value.get(),
                        max_value.get(),
                    )) {
                        None => {}
                        Some(v) => break v,
                    }
                }
            }
        }
    };
}

define_NumPrimitive_impl_nonzero!(NonZeroU8, u8);
define_NumPrimitive_impl_nonzero!(NonZeroU16, u16);
define_NumPrimitive_impl_nonzero!(NonZeroU32, u32);
define_NumPrimitive_impl_nonzero!(NonZeroU64, u64);
define_NumPrimitive_impl_nonzero!(NonZeroU128, u128);
define_NumPrimitive_impl_nonzero!(NonZeroUsize, usize);
define_NumPrimitive_impl_nonzero!(NonZeroIsize, isize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_seed() {
        assert_eq!(
            "00000000-00000000-00000000-00000000",
            Seed::from(0).to_string()
        )
    }

    #[test]
    fn string_seed_parse() {
        use std::str::FromStr;
        assert_eq!(
            Seed::from_str("10000000-01020304-12412414-09080706").expect("parse correctly"),
            Seed::from(0x10000000_01020304_12412414_09080706)
        )
    }
}
