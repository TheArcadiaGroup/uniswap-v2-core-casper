#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_snake_case)]
#[cfg(test)]
extern crate libsecp256k1;
extern crate tiny_keccak;
extern crate ewasm_api;
use libsecp256k1::{recover, Message, RecoveryId, Signature, Error::InvalidSignature};
use types::{AsymmetricType, PublicKey, account::AccountHash, bytesrepr::FromBytes};
use std::{cmp::min, convert::TryInto};
use tiny_keccak::{Hasher, Keccak};

const HASH_OFFSET: usize = 0;
const HASH_LENGTH: usize = 32;
const REC_ID_OFFSET: usize = HASH_LENGTH;
const REC_ID_LENGTH: usize = 32;
const COORD_OFFSET: usize = REC_ID_OFFSET + REC_ID_LENGTH;
const COORD_LENGTH: usize = 32;
const SIG_OFFSET: usize = COORD_OFFSET + COORD_LENGTH;
const SIG_LENGTH: usize = 32;

// converts &[u8] => &[u8; 64]
fn pop_u64(barry: &[u8]) -> &[u8; 64] {
    barry.try_into().expect("slice with incorrect length")
}

// converts &[u8] => &[u8; 32]
fn pop_u32(barry: &[u8]) -> &[u8; 32] {
    barry.try_into().expect("slice with incorrect length")
}

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
    let mut signature = *pop_u64(&[r, s].concat()[..]);
    // 3 - begin the recovery process
    let message = Message::parse(msg);
    let rec_id = RecoveryId::parse(v).unwrap();
    match Signature::parse_standard(&signature) {
        Ok(sig) => {
            let key = recover(&message, &sig, &rec_id).unwrap();
            let ret = key.serialize();
            let ret = keccak(&ret[1..65]);
            let mut output = vec![0u8; 12];
            output.extend_from_slice(&ret[12..32]);
            let pubkey = PublicKey::secp256k1_from_bytes(output.clone()).unwrap();
            let account = AccountHash::from(&pubkey);
            return account;
        }
        Err(e) => {
            eprintln!("error @ecrecover::ecrecover_sol - {}", e);
            return AccountHash::new([0u8; 32]);
        }
    }
}

/// Purpose: recovers the account hash associated with the public key from elliptic curve signature or return zero on error.
/// # Arguments
/// * `input` - A reference of an array of u8 which contains the message, signature and the recovery id.
/// # Returns
/// * A result which could be a vector of u8 or an error. 
pub fn ecrecover(input: &[u8]) -> Result<Vec<u8>, libsecp256k1::Error> {
    // *****Retrieving the message*****
    let hash_start = min(HASH_OFFSET, input.len());
    let hash_end = min(HASH_OFFSET + HASH_LENGTH, input.len());
    // h refers to the message
    let mut h = [0u8; HASH_LENGTH];
    for (i, val) in (&input[hash_start..hash_end]).iter().enumerate() {
        h[i] = *val;
    }
    // *****Retrieving the recovery id*****
    // recovery id (recid) is the last big-endian byte
    // v refers to recid
    let v = if input.len() > REC_ID_OFFSET + REC_ID_LENGTH - 1 {
        (input[REC_ID_OFFSET + REC_ID_LENGTH - 1] as i8 - 27) as u8
    } else {
        (256 - 27) as u8 /* Assume the padding would yield 0 */
    };
    if v != 0 && v != 1 {
        return Ok(vec![0u8; 0]);
    }
    // *****Retrieving the signature*****
    let sig_start = min(COORD_OFFSET, input.len());
    let sig_end = min(SIG_OFFSET + SIG_LENGTH, input.len());
    // s refers to signature
    let mut s = [0u8; 64];
    for (i, val) in (&input[sig_start..sig_end]).iter().enumerate() {
        s[i] = *val;
    }
    // *****Begin the recovery process*****
    let message = Message::parse(&h);
    let rec_id = RecoveryId::parse(v)?;
    match Signature::parse_standard(&s) {
        Ok(sig) => {
            let key = recover(&message, &sig, &rec_id)?;
            let ret = key.serialize();
            let ret = keccak(&ret[1..65]);
            let mut output = vec![0u8; 12];
            output.extend_from_slice(&ret[12..32]);
            return Ok(output)
        }
        Err(e) => return Err(e)
    }
}