#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};
use core::convert::TryInto;
use create2;
use ethereum_abi::Abi;
use parity_hash::H256;
use solid::{Address, bytesfix::{Bytes32, Bytes4}, int::Uint112};
use std::{convert::TryFrom, ops::Add};
// for contract creation code
use ethcontract_generate::ContractBuilder;
// contains evm opcodes like load, add..
use evm::Opcode;
use web3::signing::keccak256;
// I couldn't find encodePacked which is utilized in Solidity
// the difference is that encode makes calls to contracts and params are padded to 32 bytes
// encodePacked is more space-efficient and doesn't call contracts
use ethabi::Token;
use ethabi::{encode, ethereum_types::H160};

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

#[no_mangle]
extern "C" fn allPairsLength() {
    let pairs: Vec<ContractHash> = get_key("allPairs");
    ret(U256::from(pairs.len()))
}

#[no_mangle]
extern "C" fn setFeeTo() {
    let _feeTo: AccountHash = runtime::get_named_arg("feeTo");
    if (runtime::get_caller() != get_key("feeToSetter")) {
        runtime::revert(Error::UniswapV2Forbidden);
    }
    set_key("feeTo", _feeTo);
}

#[no_mangle]
extern "C" fn setFeeToSetter() {
    let _feeToSetter: AccountHash = runtime::get_named_arg("feeToSetter");
    if (runtime::get_caller() != get_key("feeToSetter")) {
        runtime::revert(Error::UniswapV2Forbidden);
    }
    set_key("feeToSetter", _feeToSetter);
}

#[no_mangle]
extern "C" fn createPair() {
    let tokenA: ContractHash = runtime::get_named_arg("tokenA");
    let tokenB: ContractHash = runtime::get_named_arg("tokenB");
    if (tokenA == tokenB) {
        runtime::revert(Error::UniswapV2IdenticalAddresses);
    }
    let (token0, token1) = if tokenA < tokenB {
        (tokenA, tokenB)
    } else {
        (tokenB, tokenA)
    };
    // in before 0 address was 0x0
    if (token0.to_string() == H256::zero().to_string()) {
        runtime::revert(Error::UniswapV2ZeroAddress);
    }
    if (pair_key(&token0, &token1).to_string() != H256::zero().to_string()) {
        runtime::revert(Error::UniswapV2PairExists);
    }
    // calling smart contracts
    // runtime::call_contract(contract_hash, entry_point_name, runtime_args)
    // generate salt
    let mut tok0_address = [0u8; 20];
    tok0_address.copy_from_slice(&token0.as_bytes());
    let mut tok1_address = [0u8; 20];
    tok1_address.copy_from_slice(&token1.as_bytes());
    let salt: Bytes32 = Bytes32(keccak256(&mut encode(&[Token::Array(vec![
        Token::Address(tok0_address.into()),
        Token::Address(tok1_address.into()),
    ])])));
    // Pair contract creation
    // 1 - set up named keys
    let mut named_keys = NamedKeys::new();
    named_keys.insert(
        "minimum_liquidity".to_string(), 
        storage::new_uref(U256::from(1000)).into()
    );
    named_keys.insert(
        "selector".to_string(), 
        storage::new_uref(*pop(&keccak256("transfer(AccountHash, U256)".as_bytes())[..])).into()
    );
    named_keys.insert(
        "factory".to_string(), 
        storage::new_uref(runtime::get_key("Factory")).into()
    );
    named_keys.insert(
        "token0".to_string(),
        storage::new_uref(token0).into(),
    );
    named_keys.insert(
        "token1".to_string(), 
        storage::new_uref(token1).into()
    );
    named_keys.insert(
        "reserve0".to_string(), 
        storage::new_uref([0u8; 14]).into()
    );
    named_keys.insert(
        "reserve1".to_string(), 
        storage::new_uref([0u8; 14]).into()
    );
    named_keys.insert(
        "blockTimestampLast".to_string(), 
        storage::new_uref(u32::MIN).into()
    );
    named_keys.insert(
        "price0CumulativeLast".to_string(), 
        storage::new_uref(U256::from(0)).into()
    );
    named_keys.insert(
        "price1CumulativeLast".to_string(), 
        storage::new_uref(U256::from(0)).into()
    );
    named_keys.insert(
        "kLast".to_string(), 
        storage::new_uref(U256::from(0)).into()
    );
    named_keys.insert(
        "unlocked".to_string(), 
        storage::new_uref(U256::from(1)).into()
    );
    // 2 - set up entry points
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(endpoint("minimum_liquidity", vec![], CLType::U256));
    entry_points.add_entry_point(endpoint("selector", vec![], CLType::Any));
    entry_points.add_entry_point(endpoint("factory", vec![], AccountHash::cl_type()));
    entry_points.add_entry_point(endpoint("token0", vec![], AccountHash::cl_type()));
    entry_points.add_entry_point(endpoint("token1", vec![], AccountHash::cl_type()));
    entry_points.add_entry_point(endpoint("reserve0", vec![], CLType::U128));
    entry_points.add_entry_point(endpoint("reserve1", vec![], CLType::U128));
    entry_points.add_entry_point(endpoint("blockTimestampLast", vec![], CLType::U32));
    entry_points.add_entry_point(endpoint("price0CumulativeLast", vec![], CLType::U256));
    entry_points.add_entry_point(endpoint("price1CumulativeLast", vec![], CLType::U256));
    entry_points.add_entry_point(endpoint("kLast", vec![], CLType::U256));
    entry_points.add_entry_point(endpoint("unlocked", vec![], CLType::U256));
    entry_points.add_entry_point(endpoint(
        "getReserves",
        vec![],
        CLType::Tuple3([Box::new(CLType::U128), Box::new(CLType::U128), Box::new(CLType::U32)]),
    ));
    entry_points.add_entry_point(endpoint(
        "mint",
        vec![
            Parameter::new("to", AccountHash::cl_type())
        ],
        CLType::U256,
    ));
    entry_points.add_entry_point(endpoint(
        "burn",
        vec![
            Parameter::new("to", AccountHash::cl_type())
        ],
        CLType::Tuple2([Box::new(CLType::U256), Box::new(CLType::U256)]),
    ));
    entry_points.add_entry_point(endpoint(
        "swap",
        vec![
            Parameter::new("amount0Out", CLType::U256),
            Parameter::new("amount1Out", CLType::U256),
            Parameter::new("to", AccountHash::cl_type()),
            Parameter::new("data", Bytes::cl_type())
        ],
        CLType::Unit,
    ));
    entry_points.add_entry_point(endpoint(
        "skim",
        vec![
            Parameter::new("to", AccountHash::cl_type())
        ],
        CLType::Unit,
    ));
    entry_points.add_entry_point(endpoint(
        "sync",
        vec![],
        CLType::Unit,
    ));
    // 3 - create the contract
    let (contract_hash, _) =
        storage::new_contract(entry_points, Some(named_keys), None, None);
    runtime::put_key("Pair", contract_hash.into());
    runtime::put_key("Pair_hash", storage::new_uref(contract_hash).into());
    // handling the pair creation by updating the storage
    set_key(&pair_key(&token0, &token1), contract_hash);
    set_key(&pair_key(&token1, &token0), contract_hash);
    let mut pairs: Vec<ContractHash> = get_key("allPairs");
    pairs.push(contract_hash);
    set_key("allPairs", pairs);
}

#[no_mangle]
pub extern "C" fn call() {
    let feeTo: AccountHash = runtime::get_named_arg("feeTo");
    let feeToSetter: AccountHash = runtime::get_named_arg("feeToSetter");
    let allPairs: Vec<ContractHash> = runtime::get_named_arg("allPairs");

    let mut named_keys = NamedKeys::new();
    named_keys.insert(
        "feeTo".to_string(), 
        storage::new_uref(feeTo).into()
    );
    named_keys.insert(
        "feeToSetter".to_string(),
        storage::new_uref(feeToSetter).into(),
    );
    named_keys.insert(
        "allPairs".to_string(), 
        storage::new_uref(allPairs).into()
    );

    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(endpoint("feeTo", vec![], AccountHash::cl_type()));
    entry_points.add_entry_point(endpoint("feeToSetter", vec![], AccountHash::cl_type()));
    entry_points.add_entry_point(endpoint(
        "allPairs",
        vec![],
        CLType::List(Box::new(ContractHash::cl_type())),
    ));
    entry_points.add_entry_point(endpoint("allPairsLength", vec![], CLType::U256));
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
            Parameter::new("tokenA", ContractHash::cl_type()),
            Parameter::new("tokenB", ContractHash::cl_type()),
        ],
        ContractHash::cl_type(),
    ));

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

fn pair_key(token0: &ContractHash, token1: &ContractHash) -> String {
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

// converts &[u8] => &[u8; 4]
fn pop(barry: &[u8]) -> &[u8; 4] {
    barry.try_into().expect("slice with incorrect length")
}

fn main() {
    println!("Hello, Factory!");
}