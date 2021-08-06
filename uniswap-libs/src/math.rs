use types::{ U256 };
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