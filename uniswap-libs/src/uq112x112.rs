// a library for handling binary fixed point numbers (https://en.wikipedia.org/wiki/Q_(number_format))
// range: [0, 2**112 - 1]
// resolution: 1 / 2**112

// Suppress all warnings from casts which overflow.
#![allow(overflowing_literals)]

use std::ops::{Div, Mul};
use solid::{encode::Encode, int::Uint112};
use types::U256;

use crate::converters::set_size_28;
// Uint224 is not a primitive type, so we need to define it here
// which enables us to implement the Mul and Div traits to access their functions.
#[derive(Clone)]
pub struct Uint224(pub [u8; 28]);

impl Mul for Uint224 {
    type Output = Uint224;
    fn mul(self, rhs: Uint224) -> Uint224 {
        // println!("self = {}", U256::from_big_endian(&(self.0)[..]));
        // println!("rhs = {}", U256::from_big_endian(&(rhs.0)[..]));
        // println!("2^112 = {}", 2u128.pow(112));
        let product = U256::from_big_endian(&(self.0)[..]) * U256::from_big_endian(&(rhs.0)[..]);
        let mut res = [0u8; 32];
        product.to_big_endian(&mut res);
        Uint224(set_size_28(&res[..]))
    }
}
impl Div for Uint224 {
    type Output = Uint224;
    fn div(self, rhs: Uint224) -> Uint224 {
        let division = U256::from_big_endian(&(self.0)[..]) / U256::from_big_endian(&(rhs.0)[..]);
        let mut res = [0u8; 32];
        division.to_big_endian(&mut res);
        Uint224(set_size_28(&res[..]))
    }
}
impl Encode for Uint224 {
    fn encode(&self) -> Vec<u8> {
        let mut value = vec![0u8; 32];
        value[32 - self.0.len()..32].copy_from_slice(&self.0);
        value
    }
}

// can't mark Q112 as a constant since I won't be able to make the required calls
// to get the Uint224 result that we desire, so I'll implement a getter function
//const Q112: Uint224 = Uint224(*set_size_28(&((2 << 112 as u32).encode())[..]));

/// # Purpose
/// returns the UQ112x112 value.
/// # Returns
/// * `UQ112x112` - the u224 constant.
fn get_q112() -> Uint224 {
    let mut v = [0u8; 32];
    //(U256::from(2).checked_pow(U256::from(112))).unwrap().to_big_endian(&mut v);
    (U256::from(2u128.pow(112))).to_big_endian(&mut v);
    return Uint224([
        v[4], v[5], v[6], v[7], v[8], v[9], v[10], v[11], v[12],
        v[13], v[14], v[15], v[16], v[17], v[18], v[19], v[20],
        v[21], v[22], v[23], v[24], v[25], v[26], v[27], v[28],
        v[29], v[30], v[31]
    ]);
}

/// # Purpose
/// encodes a `Uint112` as a `UQ112x112`.
/// # Arguments
/// * `y` - the `&Uint112` value.
/// # Returns
/// * the `Uint224` encoded value of the input.
pub fn encode(y: &Uint112) -> Uint224 {
    let e = &(y.encode())[..];
    let enc_array = [
        e[4], e[5], e[6], e[7], e[8], e[9], e[10], e[11], e[12], e[13], e[14],
        e[15], e[16], e[17], e[18], e[19], e[20], e[21], e[22], e[23], e[24], e[25],
        e[26], e[27], e[28], e[29], e[30], e[31]
    ];
    return Uint224(enc_array).mul(get_q112());
}

/// # Purpose
/// divide a `UQ112x112` by a `Uint112`, returning a `UQ112x112`.
/// # Arguments
/// * `x` - the `&Uint224` value.
/// * `y` - the `&Uint112` value.
/// # Returns
/// * the `Uint224` division result of x by y.
pub fn uqdiv(x: &Uint224, y: &Uint112) -> Uint224 {
    //let e = &(y.encode())[..];
    let e = y.0;
    let enc_array = &[[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], e].concat()[..];
    return (*x).clone().div(Uint224(set_size_28(enc_array)));
}

#[cfg(test)]
mod tests {

    use super::*;
    use types::U256;
    
    #[test]
    fn q112() {
        let expected = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let output = get_q112().0;
        assert_eq!(expected, output);
        assert_eq!(U256::from(5192296858534827628530496329220096u128), U256::from_big_endian(&expected[..]));
    }

    #[test]
    fn encode_one() {
        let input = Uint112([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        //println!("input int = {}", U256::from_big_endian(&(input.0)[..]));
        //let mut v = [0u8; 32];
        //U256::from(5192296858534827628530496329220096u128).to_big_endian(&mut v);
        //println!("q112 = {:?}", v);
        let expected = Uint224([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        //println!("expected = {:?}", expected.0);
        let output = encode(&input);
        assert_eq!(expected.0, output.0);
    }

    #[test]
    fn encode_zero() {
        let input = Uint112([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let expected = Uint224([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let output = encode(&input);
        assert_eq!(expected.0, output.0);
    }

    #[test]
    fn uqdiv_small_dividend() {
        // let mut v = [0u8; 32];
        // U256::from(2).to_big_endian(&mut v);
        let x = Uint224([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let y = Uint112([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]);
        let expected = Uint224([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let output = uqdiv(&x, &y);
        assert_eq!(expected.0, output.0);
    }

    #[test]
    fn uqdiv_large_dividend() {
        // let mut v = [0u8; 32];
        // U256::from(2).to_big_endian(&mut v);
        let x = Uint224([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]);
        let y = Uint112([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]);
        let expected = Uint224([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        let output = uqdiv(&x, &y);
        assert_eq!(expected.0, output.0);
    }

    #[test]
    #[should_panic]
    fn uqdiv_divide_by_zero() {
        // let mut v = [0u8; 32];
        // U256::from(2).to_big_endian(&mut v);
        let x = Uint224([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]);
        let y = Uint112([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        uqdiv(&x, &y);
    }
}