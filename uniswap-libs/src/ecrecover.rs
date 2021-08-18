#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_snake_case)]
#[cfg(test)]
extern crate libsecp256k1;
extern crate tiny_keccak;
use crate::converters::{set_size_32, set_size_64, u8_32_to_u32_8};
use libsecp256k1::{Error::InvalidSignature, Message, RecoveryId, Signature, curve::Scalar, recover};
use types::{AsymmetricType, PublicKey, account::AccountHash, bytesrepr::FromBytes};
use std::{cmp::min, convert::TryInto};
use tiny_keccak::{Hasher, Keccak};

pub fn keccak(input: &[u8]) -> [u8; 32] {
    let mut hash = [0u8; 32];
    let mut keccak256 = Keccak::v256();
    keccak256.update(&input);
    keccak256.finalize(&mut hash);
    hash
}

/// Purpose: recovers the account hash associated with the public key from ECDSA signature or return zero on error.
///
/// The function parameters correspond to ECDSA values of the signature.
/// # Arguments
/// * `msg` - The keccak256 hash of the original message.
/// * `v` - Final 1 byte of signature referred to as recovery id (`recid`).
/// * `r` - First 32 bytes of signature: the x-coordinate of a random curve point `kG`, modulo `n`.
/// * `s` - Second 32 bytes of signature.
/// # Returns
/// * An account hash.
pub fn ecrecover_sol(msg: &[u8; 32], v: u8, r: [u8; 32], s: [u8; 32]) -> AccountHash {
    // 1 - verify the recovery id
    if v != 0 && v != 1 {
        return AccountHash::new([0u8; 32]);
    }
    // 2 - build the signature by combining r and s
    let signature = set_size_64(&[r, s].concat()[..]);
    // 3 - begin the recovery process
    // we need to invert the [u32; 8] so that Message::parse works correctly
    // and since Message(Scalar([u32; 8])), we do the following
    let mut arr = Message::parse(msg).0.0;
    arr.reverse();
    let message = Message(Scalar(arr));
    let rec_id = RecoveryId::parse(v).unwrap();
    match Signature::parse_standard(&signature) {
        Ok(sig) => {
            let key = recover(&message, &sig, &rec_id).unwrap();
            let pubkey = PublicKey::secp256k1_from_bytes(key.serialize()).unwrap();
            let account = AccountHash::from(&pubkey);
            return account;
        }
        Err(e) => {
            eprintln!("error @ecrecover::ecrecover_sol - {}", e);
            return AccountHash::new([0u8; 32]);
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate rustc_hex;
    use crate::converters::set_size_32;

    use super::*;
    use libsecp256k1::{PublicKey, SecretKey, curve::Scalar, sign};
    use rustc_hex::FromHex;
    use types;

    #[test]
    fn ecrecover_solidty() {
        let digest = [59, 190, 92, 48, 86, 162, 86, 162, 135, 107, 163, 89, 235, 105, 205, 251, 254, 59, 53, 212, 48, 15, 215, 40, 163, 67, 63, 213, 72, 100, 210, 47];
        //let msg = Message(Scalar(u8_32_to_u32_8([1u8; 32])));
        let msg = Message(Scalar(u8_32_to_u32_8(digest)));
        let seckey = SecretKey::parse(&[1; 32]).unwrap();
        let (signature, recid) = sign(&msg, &seckey);
        let pubkey = PublicKey::from_secret_key(&seckey);
        let expected = AccountHash::from(&types::PublicKey::secp256k1_from_bytes(pubkey.serialize()).unwrap());
        let sig: [u8; 64] = signature.serialize();
        let (r, s) = sig.split_at(32);
        let output = ecrecover_sol(&digest, recid.serialize(), set_size_32(r), set_size_32(s));
        assert_eq!(recid.serialize(), 1u8);
        assert_eq!(set_size_32(r), [45, 130, 28, 82, 50, 189, 103, 184, 174, 163, 31, 62, 96, 137, 44, 132, 190, 88, 200, 120, 226, 172, 85, 208, 196, 254, 242, 121, 6, 25, 200, 215]);
        assert_eq!(set_size_32(s), [104, 149, 22, 58, 142, 184, 98, 185, 195, 115, 38, 148, 185, 156, 59, 236, 27, 70, 233, 21, 83, 17, 245, 159, 202, 156, 203, 213, 103, 110, 130, 249]);
        assert_eq!(sig, [45, 130, 28, 82, 50, 189, 103, 184, 174, 163, 31, 62, 96, 137, 44, 132, 190, 88, 200, 120, 226, 172, 85, 208, 196, 254, 242, 121, 6, 25, 200, 215, 104, 149, 22, 58, 142, 184, 98, 185, 195, 115, 38, 148, 185, 156, 59, 236, 27, 70, 233, 21, 83, 17, 245, 159, 202, 156, 203, 213, 103, 110, 130, 249]);
        assert_eq!(msg, Message(Scalar([1002331184, 1453479586, 2271978329, 3949579771, 4265293268, 806344488, 2739093461, 1214566959])));
        println!("pubkey = {:?}", pubkey);
        println!("seckey = {:?}", seckey);
        assert_eq!(expected, output);
    }

    #[test]
    fn ecrecover_solidty_zero() {
        let msg = Message(Scalar(u8_32_to_u32_8([1u8; 32])));
        let seckey = SecretKey::parse(&[1; 32]).unwrap();
        let (signature, recid) = sign(&msg, &seckey);
        let pubkey = PublicKey::from_secret_key(&seckey);
        let expected = AccountHash::from(&types::PublicKey::secp256k1_from_bytes(pubkey.serialize()).unwrap());
        let sig: [u8; 64] = signature.serialize();
        let (r, s) = sig.split_at(32);
        let output = ecrecover_sol(&[1u8; 32], recid.serialize(), set_size_32(r), set_size_32(s));
        assert_eq!(expected, output);
    }
}