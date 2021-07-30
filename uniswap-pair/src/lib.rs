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
use std::{convert::TryFrom, ops::{Add, Div, Mul, Sub}};
// I couldn't find encodePacked which is utilized in Solidity
// the difference is that encode makes calls to contracts and params are padded to 32 bytes
// encodePacked is more space-efficient and doesn't call contracts
use ethabi::{encode, ethereum_types::U128};

use contract::{contract_api::{runtime::{self, call_contract, get_blocktime, get_named_arg, put_key}, storage::{self, new_contract}}, unwrap_or_revert::UnwrapOrRevert};
use types::{ApiError, BlockTime, CLType, CLTyped, CLValue, ContractHash, Group, Parameter, RuntimeArgs, U256, URef, account::AccountHash, bytesrepr::{self, Bytes, FromBytes, ToBytes}, contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys}, runtime_args};
pub use uniswap_libs::uq112x112;

#[repr(u16)]
pub enum Error {
    UniswapV2ZeroAddress = 0,
    UniswapV2PairExists = 1,
    UniswapV2Forbidden = 2,
    UniswapV2IdenticalAddresses = 3,
    UniswapV2Overflow = 4,
    UniswapV2InsufficientLiquidityMinted = 5
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

// ***************** GETTERS - START ********************
#[no_mangle]
extern "C" fn minimum_liquidity() {
    let val: U256 = get_key("minimum_liquidity");
    ret(val)
}

#[no_mangle]
extern "C" fn selector() {
    let val: [u8; 4] = get_key("selector");
    ret(val)
}

#[no_mangle]
extern "C" fn factory() {
    let val: ContractHash = get_key("factory");
    ret(val)
}

#[no_mangle]
extern "C" fn token0() {
    let val: ContractHash = get_key("token0");
    ret(val)
}

#[no_mangle]
extern "C" fn token1() {
    let val: ContractHash = get_key("token1");
    ret(val)
}

#[no_mangle]
extern "C" fn reserve0() {
    let val: [u8; 14] = get_key("reserve0");
    ret(val)
}

#[no_mangle]
extern "C" fn reserve1() {
    let val: [u8; 14] = get_key("reserve1");
    ret(val)
}

#[no_mangle]
extern "C" fn blockTimestampLast() {
    let val: u32 = get_key("blockTimestampLast");
    ret(val)
}

#[no_mangle]
extern "C" fn price0CumulativeLast() {
    let val: U256 = get_key("price0CumulativeLast");
    ret(val)
}

#[no_mangle]
extern "C" fn price1CumulativeLast() {
    let val: U256 = get_key("price1CumulativeLast");
    ret(val)
}

#[no_mangle]
extern "C" fn kLast() {
    let val: U256 = get_key("kLast");
    ret(val)
}

#[no_mangle]
extern "C" fn getReserves() {
    let _reserve0: [u8; 14] = get_key::<[u8; 14]>("reserve0");
    let _reserve1: [u8; 14] = get_key::<[u8; 14]>("reserve1");
    let _blockTimestampLast : u32 = get_key::<u32>("blockTimestampLast ");
    let mut result: Vec<MixedType> = Vec::new();
    result.push(MixedType::Bytes(_reserve0));
    result.push(MixedType::Bytes(_reserve1));
    result.push(MixedType::Number(_blockTimestampLast));
    // result.push(_reserve0.into());
    // result.push(_reserve1.into());
    // result.push(CLType::ByteArray(_blockTimestampLast));
    ret(result)
}
// ***************** GETTERS - END ********************

#[no_mangle]
extern "C" fn mint() {
    let to: AccountHash = runtime::get_named_arg("to");
    let _reserve0: [u8; 14] = get_key::<[u8; 14]>("reserve0");
    let _reserve1: [u8; 14] = get_key::<[u8; 14]>("reserve1");
    // runtime_args should be the current contract hash, but so far this seems difficult to achieve because
    // if we are storing each contract's hash under a key named contract_name, we can't get the exact Pair contract's hash
    // since we're deploying many Pair contracts in the network, I need to find a workaround.
    let balance0: U256 = call_contract(
        get_key::<ContractHash>("token0"),
        "balance_of",
        RuntimeArgs::new());
    let balance1: U256 = call_contract(
        get_key::<ContractHash>("token1"),
        "balance_of",
        RuntimeArgs::new());
    let amount0: U256;
    let amount1: U256;
    // convert _reserve0 from [u8; 14] to U256
    match U256::from_bytes(&Uint112(_reserve0).encode()[..]) {
        Ok(res_0) => {
            amount0 = balance0.sub(res_0.0);
            // convert _reserve1 from [u8; 14] to U256
            match U256::from_bytes(&Uint112(_reserve1).encode()[..]) {
                Ok(res_1) => {
                    amount1 = balance1.sub(res_1.0);
                    let feeOn: bool = _mintFee(Uint112(_reserve0), Uint112(_reserve1));
                    // return value -> liquidity
                    let mut liquidity: U256;
                    let _total_supply: U256 = get_key::<U256>("total_supply");
                    if (_total_supply == U256::from(0)) {
                        liquidity = (amount0.mul(amount1)).integer_sqrt().sub(get_key::<U256>("minimum_liquidity"));
                        //_mint(AccountHash([0u8; 32]), get_key::<U256>("minimum_liquidity"))
                    }
                    else {
                        liquidity = (amount0.mul(_total_supply)).div(res_0.0);
                        liquidity = liquidity.min((amount1.mul(_total_supply)).div(res_1.0));
                    }
                    if (liquidity <= U256::from(0)) {
                        runtime::revert(Error::UniswapV2InsufficientLiquidityMinted);
                    }
                    //_mint(to, liquidity);
                    _update(balance0, balance1, Uint112(_reserve0), Uint112(_reserve1));
                    if (feeOn) {
                        set_key::<U256>("kLast", res_0.0.mul(res_1.0));
                    }
                }
                Err(e) => println!("Error @pair::mint - {}", e)
            }
        }
        Err(e) => println!("Error @pair::mint - {}", e)
    }
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
        match U256::from_bytes(&(uq112x112::uqdiv(&uq112x112::encode(&_reserve1), &_reserve0)).encode()[..]) {
            Ok(res) => price0CumulativeLast += res.0,
            Err(e) => println!("Error @pair::_update - {}", e)
        }
        set_key::<U256>("price0CumulativeLast", price0CumulativeLast);
        let mut price1CumulativeLast: U256 = get_key::<U256>("price1CumulativeLast");
        match U256::from_bytes(&(uq112x112::uqdiv(&uq112x112::encode(&_reserve0), &_reserve1)).encode()[..]) {
            Ok(res) => price1CumulativeLast += res.0,
            Err(e) => println!("Error @pair::_update - {}", e)
        }
        set_key::<U256>("price1CumulativeLast", price1CumulativeLast);
    }
    set_key::<[u8; 14]>("reserve0", *pop_u112(&(balance0.as_u128().encode())[..]));
    set_key::<[u8; 14]>("reserve1", *pop_u112(&(balance1.as_u128().encode())[..]));
    set_key::<u32>("blockTimestampLast", blockTimestamp);
}

fn _mintFee(_reserve0: Uint112, _reserve1: Uint112) -> bool {
    let feeTo: AccountHash = call_contract(get_key("factory"), "feeTo", RuntimeArgs::new());
    let feeOn: bool = feeTo.value() != [0u8; 32];
    let _kLast: U256 = get_key("kLast");
    if (feeOn) {
        if (_kLast != U256::from(0)) {
            let rootK: U256;
            // convert _reserve0 from Uint112 to U256
            match U256::from_bytes(&(_reserve0.encode())[..]) {
                Ok(result_1) => {
                    // convert _reserve1 from Uint112 to U256
                    match U256::from_bytes(&(_reserve0.encode())[..]) {
                        Ok(result_2) => {
                            match result_1.0.checked_mul(result_2.0) {
                                Some(x) => {
                                    rootK = x.integer_sqrt();
                                    let rootKLast: U256 = _kLast.integer_sqrt();
                                    if (rootK > rootKLast) {
                                        let numerator: U256 = get_key::<U256>("total_supply").mul(rootK.sub(rootKLast));
                                        let denominator: U256 = rootK.mul(U256::from(5)).add(rootKLast);
                                        let liquidity: U256 = numerator.div(denominator);
                                        // need to include _mint fct from UniswapV2ERC20
                                        //if (liquidity > U256::from(0)) _mint(feeTo, liquidity)
                                        //uniswap_erc20::
                                    }
                                }
                                None => println!("Multiplication using 'checked_mul' failed due to Overflow")
                            }
                        }
                        Err(e) => println!("Error @pair::_mintFee - {}", e)
                    }
                },
                Err(e) => println!("Error @pair::_mintFee - {}", e)
            }
        }
    }
    else if (_kLast != U256::from(0)) {
        set_key::<U256>("kLast", U256::from(0));
    }
    return feeOn;
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

fn pop_u112(barry: &[u8]) -> &[u8; 14] {
    barry.try_into().expect("slice with incorrect length")
}