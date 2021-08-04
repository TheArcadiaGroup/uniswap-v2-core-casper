// a library for handling binary fixed point numbers (https://en.wikipedia.org/wiki/Q_(number_format))
// range: [0, 2**112 - 1]
// resolution: 1 / 2**112

// Suppress all warnings from casts which overflow.
#![allow(overflowing_literals)]

use std::{convert:: TryInto, ops::{Div, Mul}};
use solid::{encode::Encode, int::Uint112};
use types::U256;

// Uint224 is not a primitive type, so we need to define it here
// which enables us to implement the Mul and Div traits to access their functions
pub struct Uint224(pub [u8; 28]);

impl Mul for Uint224 {
    type Output = Uint224;
    fn mul(self, rhs: Uint224) -> Uint224 {
        return self.mul(rhs);
    }
}
impl Div for Uint224 {
    type Output = Uint224;
    fn div(self, rhs: Uint224) -> Uint224 {
        return self.div(rhs);
    }
}
impl Encode for Uint224 {
    fn encode(&self) -> Vec<u8> {
        return self.encode();
    }
}
impl Clone for Uint224 {
    fn clone(&self) -> Self {
        return self.clone();
    }
}

// **** Uint112 => Uint224 steps: *****
// 1 - convert Uint112 => Vec[u8] with encode()
// 2 - convert Vec[u8] => &[u8] using & and [..]
// 3 - call pop_u28() fct to convert &[u8] => &[u8; 28]
// 4 - Uint224() converts [u8; 28] => Uint224

// can't mark Q112 as a constant since I won't be able to make the required calls
// to get the Uint224 result that we desire, so I'll implement a getter function
//const Q112: Uint224 = Uint224(*pop_u28(&((2 << 112 as u32).encode())[..]));

fn get_q112() -> Uint224 {
    return Uint224(*pop_u28(&((2i32.pow(112)).encode())[..]));
}

// converts &[u8] => &[u8; 28]
fn pop_u28(barry: &[u8]) -> &[u8; 28] {
    barry.try_into().expect("slice with incorrect length")
}

// encode a uint112 as a UQ112x112
pub fn encode(y: &Uint112) -> Uint224 {
    return Uint224(*pop_u28(&(y.encode())[..])).mul(get_q112());
}

// divide a UQ112x112 by a uint112, returning a UQ112x112
pub fn uqdiv(x: &Uint224, y: &Uint112) -> Uint224 {
    return (*x).clone().div(Uint224(*pop_u28(&(y.encode())[..])));
}