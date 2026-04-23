//! char based generator types

use super::super::rand::R;
use super::base::Generator;
use super::numerical::range;

/// Generate ASCII char
#[derive(Clone, Copy)]
pub struct AsciiChar;

impl Generator for AsciiChar {
    type Item = char;

    fn generate(&self, r: &mut R) -> Self::Item {
        range(0x20..0x7f)
            .map(|n| std::char::from_u32(n).unwrap())
            .generate(r)
    }
}

/// generate ASCII char
pub fn ascii() -> AsciiChar {
    AsciiChar
}

/// Generate Digits char
#[derive(Clone, Copy)]
pub struct DigitsChar;

impl Generator for DigitsChar {
    type Item = char;

    fn generate(&self, r: &mut R) -> Self::Item {
        range('0'..'9').generate(r)
    }
}

/// generate ASCII char
pub fn digits() -> DigitsChar {
    DigitsChar
}
