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
use solid::{Address, bytesfix::{Bytes32, Bytes4}, encode::Encode, int::{Uint112, Uint224}};
use std::{convert::TryFrom, ops::Add};
// I couldn't find encodePacked which is utilized in Solidity
// the difference is that encode makes calls to contracts and params are padded to 32 bytes
// encodePacked is more space-efficient and doesn't call contracts
use ethabi::{encode, ethereum_types::U128};

use contract::{contract_api::{runtime::{self, get_blocktime, get_named_arg}, storage::{self, new_contract}}, unwrap_or_revert::UnwrapOrRevert};
use types::{ApiError, BlockTime, CLType, CLTyped, CLValue, ContractHash, Group, Parameter, RuntimeArgs, U256, URef, account::AccountHash, bytesrepr::{self, Bytes, FromBytes, ToBytes}, contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys}, runtime_args};
pub use uniswap_libs;

#[repr(u16)]
pub enum Error {
    UniswapV2ZeroAddress = 0,
    UniswapV2PairExists = 1,
    UniswapV2Forbidden = 2, // 65538
    UniswapV2IdenticalAddresses = 3,
    UniswapV2Overflow = 4
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

fn _safeTransfer(token: ContractHash, to: AccountHash, value: U256) {
    // 1 - prepare runtime args for the transfer function
    let mut transfer_args: RuntimeArgs = RuntimeArgs::new();
    transfer_args.insert("recipient", to).expect("Couldn't insert the recipient argument @_safeTransfer");
    transfer_args.insert("amount", value).expect("Couldn't insert the amount argument @_safeTransfer");
    // 2 - call the token's contract transfer function
    runtime::call_contract::<()>(token, "transfer", transfer_args);
    // In Solidity, we catch a revert here, but in our case the revert will be thrown
    // from within the contract call
}

fn _update(balance0: U256, balance1: U256, _reserve0: Uint112, _reserve1: Uint112) {
    if (balance0 > U256::from(2i32.pow(112) - 1) || balance1 > U256::from(2i32.pow(112) - 1)) {
        runtime::revert(Error::UniswapV2Overflow);
    }
    // assign a value to blockTimestamp just to avoid the error "use of possibly-uninitialized variable"
    let mut blockTimestamp = u32::MAX;
    // Here, we are sure that checked_rem() will result in Some() and not None
    match u64::from(get_blocktime()).checked_rem(2i32.pow(32) as u64) {
        Some(res) => blockTimestamp = res as u32,
        None => println!("Cannot divide by zero @pair::_update")
    }
    let timeElapsed: u32 = blockTimestamp - get_key::<u32>("blockTimestampLast");
    if (timeElapsed > u32::MIN && u128::from_be_bytes(*pop(&(_reserve0.encode())[..])) != u128::MIN && u128::from_be_bytes(*pop(&(_reserve1.encode())[..])) != u128::MIN) {
        let mut price0CumulativeLast: U256 = get_key::<U256>("price0CumulativeLast");
        //let x: Uint224 = uniswap_libs::uq112x112::encode(_reserve0);
    }
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

fn pop(barry: &[u8]) -> &[u8; 16] {
    barry.try_into().expect("slice with incorrect length")
}
fn main() {
    println!("Hello, Pair!");
}