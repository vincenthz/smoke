//! string based generator types

use super::super::rand::R;
use super::base::{BoxGenerator, Generator};
use super::numerical::range;

/// Generate String containing only ASCII characters
pub struct AsciiString(BoxGenerator<usize>);

impl Generator for AsciiString {
    type Item = String;

    fn gen(&self, r: &mut R) -> Self::Item {
        let sz = self.0.gen(&mut r.sub());
        let mut chars_r = r.sub();
        let mut out = Vec::with_capacity(sz);
        let ascii_range = range(0x20..0x7f).map(|n| std::char::from_u32(n).unwrap());
        for _ in 0..sz {
            out.push(ascii_range.gen(&mut chars_r))
        }
        out.iter().collect()
    }
}

/// generate ASCII string of size specified by the generator in parameter
///
/// ```
/// use smoke::generator::{string::ascii, constant, range};
///
/// let fixed_size_string = ascii(constant(12));
/// let small_string = ascii(range(1..8));
/// ```
pub fn ascii<SZ: Generator<Item = usize> + 'static>(size: SZ) -> AsciiString {
    AsciiString(size.into_boxed())
}

/// Generate String of size specified by the first generator and of character range
/// specified by the second generator
pub struct StringGenerator(BoxGenerator<usize>, BoxGenerator<char>);

impl Generator for StringGenerator {
    type Item = String;

    fn gen(&self, r: &mut R) -> Self::Item {
        let sz = self.0.gen(&mut r.sub());
        let mut chars_r = r.sub();
        let mut out = Vec::with_capacity(sz);
        for _ in 0..sz {
            out.push(self.1.gen(&mut chars_r))
        }
        out.iter().collect()
    }
}

/// generate arbitary string of size specified by the first generator in parameter
/// and character set specified by the character generator
///
/// ```
/// use smoke::generator::{string::string, constant, range};
///
/// let four_digits_string = string(constant(4), range('0'..'9'));
/// let small_letters = string(range(1..8), range('a'..'z'));
/// ```
pub fn string<SZ: Generator<Item = usize> + 'static, C: Generator<Item = char> + 'static>(
    size: SZ,
    chars: C,
) -> StringGenerator {
    StringGenerator(size.into_boxed(), chars.into_boxed())
}
