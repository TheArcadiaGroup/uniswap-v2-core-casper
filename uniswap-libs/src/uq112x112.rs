// a library for handling binary fixed point numbers (https://en.wikipedia.org/wiki/Q_(number_format))
// range: [0, 2**112 - 1]
// resolution: 1 / 2**112

// Suppress all warnings from casts which overflow.
#![allow(overflowing_literals)]

use std::{convert:: TryInto, ops::{Div, Mul}};
use solid::{encode::Encode, int::Uint112};

// Uint224 is not a primitive type, so we need to define it here
// which enables us to implement the Mul and Div traits to access their functions.
#[derive(Clone)]
pub struct Uint224(pub [u8; 28]);

impl Mul for Uint224 {
    type Output = Uint224;
    fn mul(self, rhs: Uint224) -> Uint224 {
        let mut res: [u8; 28] = [0u8; 28];
        for i in 0..self.0.len() {
            res[i] = self.0[i] * rhs.0[i];
        }
        Uint224(res)
    }
}
impl Div for Uint224 {
    type Output = Uint224;
    fn div(self, rhs: Uint224) -> Uint224 {
        let mut res: [u8; 28] = [0u8; 28];
        for i in 0..self.0.len() {
            res[i] = self.0[i] / rhs.0[i];
        }
        Uint224(res)
    }
}
impl Encode for Uint224 {
    fn encode(&self) -> Vec<u8> {
        let mut value = vec![0u8; 32];
        value[32 - self.0.len()..32].copy_from_slice(&self.0);
        value
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

/// # Purpose
/// returns the UQ112x112 value.
/// # Returns
/// * `UQ112x112` - the u224 constant.
fn get_q112() -> Uint224 {
    return Uint224(*pop_u28(&((2i32.pow(112)).encode())[..]));
}

/// # Purpose
/// converts an `&[u8]` to a `&[u8; 28]`.
/// # Arguments
/// * `primitive` - the `&[u8]` value.
/// # Returns
/// * the `&[u8; 28]` equivalent of the given input.
fn pop_u28(primitive: &[u8]) -> &[u8; 28] {
    primitive.try_into().expect("slice with incorrect length")
}

/// # Purpose
/// encodes a `Uint112` as a `UQ112x112`.
/// # Arguments
/// * `y` - the `&Uint112` value.
/// # Returns
/// * the `Uint224` encoded value of the input.
pub fn encode(y: &Uint112) -> Uint224 {
    return Uint224(*pop_u28(&(y.encode())[..])).mul(get_q112());
}

/// # Purpose
/// divide a `UQ112x112` by a `Uint112`, returning a `UQ112x112`.
/// # Arguments
/// * `x` - the `&Uint224` value.
/// * `y` - the `&Uint112` value.
/// # Returns
/// * the `Uint224` division result of x by y.
pub fn uqdiv(x: &Uint224, y: &Uint112) -> Uint224 {
    return (*x).clone().div(Uint224(*pop_u28(&(y.encode())[..])));
}