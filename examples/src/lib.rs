#[cfg(test)]
mod tests {
    use smoke::generator::{num, range_bounds};
    use smoke::property;
    use smoke_macros::smoketest;
    use smoke::Property;

    #[smoketest{a: num::<u32>()}]
    fn test1(a: u32) {
        property::equal(a, a)
    }

    #[smoketest{a: num::<u32>(), b: num::<u32>() }]
    fn test2(a: u32, b: u32) {
        property::equal(a + b, b + a)
    }

    #[smoketest{r: range_bounds::<u8, std::ops::RangeInclusive<u8>>(20..=40)}]
    fn test3(r: u32) {
        property::less_equal(20, r).and(property::less_equal(r, 40))
    }

    #[smoketest{r: range_bounds(..20u32)}]
    fn test4(r: u32) {
        property::less(r, 20)
    }
}
