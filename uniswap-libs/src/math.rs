use types::{ U256 };
//use integer_sqrt;

fn min(x: U256, y: U256) -> U256 {
    return std::cmp::min(x, y);
}

fn sqrt(x: U256) -> U256 {
    return x.integer_sqrt();
}