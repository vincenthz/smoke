#[cfg(test)]
mod tests {
    use smoke::generator::num;
    use smoke::property;
    use smoke_macros::smoketest;

    #[smoketest{a: num::<u32>()}]
    fn test1(a: u32) {
        property::equal(a, a)
    }

    #[smoketest{a: num::<u32>(), b: num::<u32>() }]
    fn test2(a: u32, b: u32) {
        property::equal(a + b, b + a)
    }
}
