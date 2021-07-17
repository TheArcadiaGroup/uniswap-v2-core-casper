#![no_main]
#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};
use solid::Address;
use core::convert::TryInto;

use contract::{contract_api::{runtime::{self, get_named_arg}, storage}, unwrap_or_revert::UnwrapOrRevert};
use types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys},
    runtime_args, CLType, CLTyped, CLValue, Group, Parameter, RuntimeArgs, URef, U256,
};

#[no_mangle]
extern "C" fn allPairsLength() {
    let pairs: [&AccountHash] = get_key("allPairs");
    ret(pairs.len())
}

#[no_mangle]
#[require(runtime::get_caller() == get_key("feeToSetter"), "UniswapV2: FORBIDDEN")]
extern "C" fn setFeeTo() {
    let _feeTo: AccountHash = runtime::get_named_arg("feeTo");
    set_key("feeTo", _feeTo);
}

#[no_mangle]
#[require(runtime::get_caller() == get_key("feeToSetter"), "UniswapV2: FORBIDDEN")]
extern "C" fn setFeeToSetter() {
    let _feeToSetter: AccountHash = runtime::get_named_arg("feeToSetter");
    set_key("feeToSetter", _feeToSetter);
}

#[no_mangle]
#[require(tokenA != tokenB, "UniswapV2: IDENTICAL_ADDRESSES")]
#[require(tokenA != '0x0' && tokenB != '0x0', "UniswapV2: ZERO_ADDRESS")]
#[require(pair_key(&tokenA, &tokenB) == '0x0', "UniswapV2: PAIR_EXISTS")]
extern "C" fn createPair() {
    let tokenA: AccountHash = runtime::get_named_arg("tokenA");
    let tokenB: AccountHash = runtime::get_named_arg("tokenB");
    let (mut token0, mut token1) = if tokenA < tokenB {(tokenA, tokenB)} else {(tokenB, tokenA)};
    // generate pair address
}

#[no_mangle]
pub extern "C" fn call() {
    let feeTo: AccountHash = runtime::get_named_arg("feeTo");
    let feeToSetter: AccountHash = runtime::get_named_arg("feeToSetter");
    let allPairs: [AccountHash] = runtime::get_named_arg("allPairs");

    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(endpoint("feeTo", vec![], AccountHash::cl_type()));
    entry_points.add_entry_point(endpoint("feeToSetter", vec![], AccountHash::cl_type()));
    entry_points.add_entry_point(endpoint("allPairs", vec![], [AccountHash::cl_type()]));
    entry_points.add_entry_point(endpoint(
        "allPairsLength",
        vec![],
        CLType::U256,
    ));
    entry_points.add_entry_point(endpoint(
        "setFeeTo",
        vec![Parameter::new("feeTo", AccountHash::cl_type())],
        CLType::Unit,
    ));
    entry_points.add_entry_point(endpoint(
        "setFeeToSetter",
        vec![Parameter::new("feeToSetter", AccountHash::cl_type())],
        CLType::Unit,
    ));
    entry_points.add_entry_point(endpoint(
        "createPair",
        vec![
            Parameter::new("tokenA", AccountHash::cl_type()),
            Parameter::new("tokenB", AccountHash::cl_type()),
        ],
        AccountHash::cl_type(),
    ));

    let mut named_keys = NamedKeys::new();
    named_keys.insert("feeTo".to_string(), storage::new_uref(feeTo).into());
    named_keys.insert("feeToSetter".to_string(), storage::new_uref(feeToSetter).into());
    named_keys.insert("allPairs", storage::new_uref(allPairs).into());

    let (contract_hash, _) =
        storage::new_locked_contract(entry_points, Some(named_keys), None, None);
    runtime::put_key("Factory", contract_hash.into());
    runtime::put_key("Factory_hash", storage::new_uref(contract_hash).into());
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

fn pair_key(token0: &AccountHash, token1: &AccountHash) -> AccountHash {
    format!("getPair_{}_{}", token0, token1)
}

fn endpoint(name: &str, param: Vec<Parameter>, ret: CLType) -> EntryPoint {
    EntryPoint::new(
        String::from(name),
        param,
        ret,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}