// a library for handling binary fixed point numbers (https://en.wikipedia.org/wiki/Q_(number_format))
// range: [0, 2**112 - 1]
// resolution: 1 / 2**112

use solid::int::{self, Uint112, Uint224};

const Q112: Uint224 = 2**112;

// encode a uint112 as a UQ112x112
fn encode(y: Uint112) -> Uint224 {
    return Uint224(y) * Q112;
}

// divide a UQ112x112 by a uint112, returning a UQ112x112
fn uqdiv(x: Uint224, y: Uint112) -> Uint224 {
    return x / Uint224(y);
}