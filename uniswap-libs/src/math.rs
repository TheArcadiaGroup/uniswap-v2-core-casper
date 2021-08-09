use types::U256;
//use integer_sqrt;

#[allow(dead_code)]
/// # Purpose
/// returns the minimun of an of two given `U256` values.
/// # Arguments
/// * `x` - the first `U256` value.
/// * `y` - the second `U256` value.
/// # Returns
/// * the `U256` minimum value between the two inputs.
fn min(x: U256, y: U256) -> U256 {
    return std::cmp::min(x, y);
}

#[allow(dead_code)]
/// # Purpose
/// returns the square root of a given `U256` value.
/// # Arguments
/// * `x` - the `U256` value.
/// # Returns
/// * the `U256` square root of the input.
fn sqrt(x: U256) -> U256 {
    return x.integer_sqrt();
}

#[cfg(test)]
mod tests {

    use types::U256;
    use super::*;
    
    #[test]
    fn min_test() {
        let input_1 = U256::from(0);
        let input_2 = U256::from(1);
        let expected = U256::from(0);
        let output = min(input_1, input_2);
        assert_eq!(expected, output);
    }

    #[test]
    fn sqrt_test() {
        let input = U256::from(4);
        let expected = U256::from(2);
        let output = sqrt(input);
        assert_eq!(expected, output);
    }
}