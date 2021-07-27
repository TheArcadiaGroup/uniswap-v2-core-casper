#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};
use core::convert::TryInto;
use parity_hash::H256;
use solid::{Address, bytesfix::{Bytes32, Bytes4}, int::Uint112};
use std::{convert::TryFrom, ops::Add};
// I couldn't find encodePacked which is utilized in Solidity
// the difference is that encode makes calls to contracts and params are padded to 32 bytes
// encodePacked is more space-efficient and doesn't call contracts
use ethabi::{encode, ethereum_types::U128};

use contract::{contract_api::{runtime::{self, get_named_arg}, storage::{self, new_contract}}, unwrap_or_revert::UnwrapOrRevert};
use types::{ApiError, CLType, CLTyped, CLValue, ContractHash, Group, Parameter, RuntimeArgs, U256, URef, account::AccountHash, bytesrepr::{self, Bytes, FromBytes, ToBytes}, contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys}, runtime_args};

#[repr(u16)]
pub enum Error {
    UniswapV2ZeroAddress = 0,
    UniswapV2PairExists = 1,
    UniswapV2Forbidden = 2, // 65538
    UniswapV2IdenticalAddresses = 3,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}

enum MixedType {
    Bytes([u8; 14]),
    Number(u32),
}

impl CLTyped for MixedType {
    fn cl_type() -> CLType { todo!() }
}

impl ToBytes for MixedType {
    fn to_bytes(&self) -> std::result::Result<Vec<u8>, bytesrepr::Error> { todo!() }
    fn serialized_length(&self) -> usize { todo!() }
}

#[no_mangle]
extern "C" fn getReserves() {
    let _reserve0: [u8; 14] = get_key("reserve0");
    let _reserve1: [u8; 14] = get_key("reserve1");
    let _blockTimestampLast : u32 = get_key("blockTimestampLast ");
    let mut result: Vec<MixedType> = Vec::new();
    result.push(MixedType::Bytes(_reserve0));
    result.push(MixedType::Bytes(_reserve1));
    result.push(MixedType::Number(_blockTimestampLast));
    // result.push(_reserve0.into());
    // result.push(_reserve1.into());
    // result.push(CLType::ByteArray(_blockTimestampLast));
    ret(result)
}

fn ret<T: CLTyped + ToBytes>(value: T) {
    runtime::ret(CLValue::from_t(value).unwrap_or_revert())
}

fn get_key<T: FromBytes + CLTyped + Default>(name: &str) -> T {
    match runtime::get_key(name) {
        None => Default::default(),
        Some(value) => {
            let key = value.try_into().unwrap_or_revert();
            storage::read(key).unwrap_or_revert().unwrap_or_revert()
        }
    }
}

fn set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
    match runtime::get_key(name) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, value);
        }
        None => {
            let key = storage::new_uref(value).into();
            runtime::put_key(name, key);
        }
    }
}
fn main() {
    println!("Hello, Pair!");
}