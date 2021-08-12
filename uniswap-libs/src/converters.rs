// Suppress all warnings from casts which overflow.
#![allow(overflowing_literals)]

use std::convert:: TryInto;

// **** Uint112 => Uint224 steps: *****
// 1 - convert Uint112 => Vec[u8] with encode()
// 2 - convert Vec[u8] => &[u8] using & and [..]
// 3 - call set_size_28() fct to convert &[u8] => [u8; 28]
// 4 - Uint224() converts [u8; 28] => Uint224

/// # Purpose
/// converts an `&[u8]` to a `[u8; 4]`.
/// # Arguments
/// * `primitive` - the `&[u8]` value.
/// # Returns
/// * the `[u8; 4]` equivalent of the given input.
pub fn set_size_4(primitive: &[u8]) -> [u8; 4] {
    let mut x = primitive.to_vec();
    x.reverse();
    x.truncate(4);
    x.reverse();
    x.as_slice().try_into().expect("slice with incorrect length")
}

/// # Purpose
/// converts an `&[u8]` to a `[u8; 14]`.
/// # Arguments
/// * `primitive` - the `&[u8]` value.
/// # Returns
/// * the `[u8; 14]` equivalent of the given input.
pub fn set_size_14(primitive: &[u8]) -> [u8; 14] {
    let mut x = primitive.to_vec();
    x.reverse();
    x.truncate(14);
    x.reverse();
    x.as_slice().try_into().expect("slice with incorrect length")
}

/// # Purpose
/// converts an `&[u8]` to a `[u8; 16]`.
/// # Arguments
/// * `primitive` - the `&[u8]` value.
/// # Returns
/// * the `[u8; 16]` equivalent of the given input.
pub fn set_size_16(primitive: &[u8]) -> [u8; 16] {
    let mut x = primitive.to_vec();
    x.reverse();
    x.truncate(16);
    x.reverse();
    x.as_slice().try_into().expect("slice with incorrect length")
}

/// # Purpose
/// converts an `&[u8]` to a `[u8; 28]`.
/// # Arguments
/// * `primitive` - the `&[u8]` value.
/// # Returns
/// * the `[u8; 28]` equivalent of the given input.
pub fn set_size_28(primitive: &[u8]) -> [u8; 28] {
    let mut x = primitive.to_vec();
    x.reverse();
    x.truncate(28);
    x.reverse();
    x.as_slice().try_into().expect("slice with incorrect length")
}

/// # Purpose
/// converts an `&[u8]` to a `[u8; 32]`.
/// # Arguments
/// * `primitive` - the `&[u8]` value.
/// # Returns
/// * the `[u8; 32]` equivalent of the given input.
pub fn set_size_32(primitive: &[u8]) -> [u8; 32] {
    primitive.try_into().expect("slice with incorrect length")
}

/// # Purpose
/// converts an `&[u8]` to a `[u8; 64]`.
/// # Arguments
/// * `primitive` - the `&[u8]` value.
/// # Returns
/// * the `[u8; 64]` equivalent of the given input.
pub fn set_size_64(primitive: &[u8]) -> [u8; 64] {
    primitive.try_into().expect("slice with incorrect length")
}

/// # Purpose
/// converts an `types::U256` to a `ethabi::ethereum_types::U256`.
/// # Arguments
/// * `primitive` - the `types::U256` value which is a wrapper around `[u8; 32]`.
/// # Returns
/// * the `ethabi::ethereum_types::U256` equivalent of the given input which is a wrapper around `[u64; 4]`.
// pub fn to_ethabi_u256(primitive: U256) -> ethabi::ethereum_types::U256 {
//     let mut v = [0u8; 32];
//     primitive.to_little_endian(&mut v);
//     ethabi::ethereum_types::U256([
//         u64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]),
//         u64::from_be_bytes([v[8], v[9], v[10], v[11], v[12], v[13], v[14], v[15]]),
//         u64::from_be_bytes([v[16], v[17], v[18], v[19], v[20], v[21], v[22], v[23]]),
//         u64::from_be_bytes([v[24], v[25], v[26], v[27], v[28], v[29], v[30], v[31]])
//     ])
// }

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn size_4() {
        let input = &vec![1, 2, 3, 4][..];
        let expected = vec![1, 2, 3, 4];
        let output = set_size_4(input);
        assert_eq!(expected, output);
    }

    #[test]
    fn size_14() {
        let input = &vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14][..];
        let expected = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        let output = set_size_14(input);
        assert_eq!(expected, output);
    }

    #[test]
    fn size_16() {
        let input = &vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16][..];
        let expected = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let output = set_size_16(input);
        assert_eq!(expected, output);
    }

    #[test]
    fn size_28() {
        let input = &vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
            18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28
        ][..];
        let expected = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
            18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28
        ];
        let output = set_size_28(input);
        assert_eq!(expected, output);
    }

    #[test]
    fn size_32() {
        let input = &vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
            18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
        ][..];
        let expected = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
            18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
        ];
        let output = set_size_32(input);
        assert_eq!(expected, output);
    }

    #[test]
    fn size_64() {
        let input = &vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
            18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
            33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
            48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62,
            63, 64
        ][..];
        let expected = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
            18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
            33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
            48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62,
            63, 64
        ];
        let output = set_size_64(input);
        assert_eq!(expected, output);
    }

    // #[test]
    // fn ethabi_u256() {
    //     let input = U256::from_little_endian(&vec![
    //         1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
    //         18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
    //     ][..]);
    //     println!("input = {}", input);
    //     let expected = ethabi::ethereum_types::U256([
    //         u64::from_be_bytes([1, 2, 3, 4, 5, 6, 7, 8]),
    //         u64::from_be_bytes([9, 10, 11, 12, 13, 14, 15, 16]),
    //         u64::from_be_bytes([17, 18, 19, 20, 21, 22, 23, 24]),
    //         u64::from_be_bytes([25, 26, 27, 28, 29, 30, 31, 32])
    //     ]);
    //     let output = to_ethabi_u256(input);
    //     assert_eq!(expected, output);
    // }
}