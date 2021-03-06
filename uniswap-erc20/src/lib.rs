#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};
use core::convert::TryInto;
use solid::{Address, bytesfix::{Bytes32, Bytes4}, int::Uint112};
use renvm_sig::keccak256;
use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{ApiError, CLType, CLTyped, CLValue, ContractHash, Group, Parameter, RuntimeArgs, U256, URef, account::AccountHash, bytesrepr::{FromBytes, ToBytes}, contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys}, runtime_args};
use elliptic_curve;
use uniswap_libs::ecrecover;

pub enum Error {
    UniswapV2ZeroAddress = 0,
    UniswapV2PairExists = 1,
    UniswapV2Forbidden = 2,
    UniswapV2IdenticalAddresses = 3,
    UniswapV2Overflow = 4,
    UniswapV2InsufficientLiquidityMinted = 5,
    UniswapV2InsufficientLiquidityBurned = 6,
    UniswapV2Locked = 7,
    UniswapV2InsufficientInputAmount = 8,
    UniswapV2InsufficientOutputAmount = 9,
    UniswapV2InsufficientLiquidity = 10,
    UniswapV2InvalidTo = 11,
    UniswapV2K = 12,
    UniswapV2Expired = 13,
    UniswapV2InvalidSignature = 14
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}

#[cfg(not(feature = "no_name"))]
#[no_mangle]
pub extern "C" fn name() {
    let val: String = get_key("name");
    ret(val)
}

#[cfg(not(feature = "no_symbol"))]
#[no_mangle]
pub extern "C" fn symbol() {
    let val: String = get_key("symbol");
    ret(val)
}

#[cfg(not(feature = "no_decimals"))]
#[no_mangle]
pub extern "C" fn decimals() {
    let val: u8 = get_key("decimals");
    ret(val)
}

#[cfg(not(feature = "no_total_supply"))]
#[no_mangle]
pub extern "C" fn total_supply() {
    let val: U256 = get_key("total_supply");
    ret(val)
}

#[cfg(not(feature = "no_balance_of"))]
#[no_mangle]
pub extern "C" fn balance_of() {
    let owner: AccountHash = runtime::get_named_arg("owner");
    let val: U256 = get_key(&balance_key(&owner));
    ret(val)
}

#[cfg(not(feature = "no_allowance"))]
#[no_mangle]
pub extern "C" fn allowance() {
    let owner: AccountHash = runtime::get_named_arg("owner");
    let spender: AccountHash = runtime::get_named_arg("spender");
    let val: U256 = get_key(&allowance_key(&owner, &spender));
    ret(val)
}

#[cfg(not(feature = "no_domain_separator"))]
#[no_mangle]
pub extern "C" fn domain_separator() {
    let val: [u8; 32] = get_key("domain_separator");
    ret(val)
}

#[cfg(not(feature = "no_permit_typehash"))]
#[no_mangle]
pub extern "C" fn permit_typehash() {
    let val: [u8; 32] = get_key("permit_typehash");
    ret(val)
}

#[cfg(not(feature = "no_nonces"))]
#[no_mangle]
pub extern "C" fn nonces() {
    let owner: AccountHash = runtime::get_named_arg("owner");
    let val: U256 = get_key(&nonce_key(&owner));
    ret(val)
}

#[cfg(not(feature = "no_approve"))]
#[no_mangle]
pub extern "C" fn approve() {
    let spender: AccountHash = runtime::get_named_arg("spender");
    let amount: U256 = runtime::get_named_arg("amount");
    _approve(runtime::get_caller(), spender, amount);
}

#[cfg(not(feature = "no_transfer"))]
#[no_mangle]
pub extern "C" fn transfer() {
    let recipient: AccountHash = runtime::get_named_arg("recipient");
    let amount: U256 = runtime::get_named_arg("amount");
    _transfer(runtime::get_caller(), recipient, amount);
}

#[cfg(not(feature = "no_transfer_from"))]
#[no_mangle]
pub extern "C" fn transfer_from() {
    let owner: AccountHash = runtime::get_named_arg("owner");
    let recipient: AccountHash = runtime::get_named_arg("recipient");
    let amount: U256 = runtime::get_named_arg("amount");
    _transfer_from(owner, recipient, amount);
}

#[cfg(not(feature = "no_permit"))]
#[no_mangle]
pub extern "C" fn permit() {
    let owner: AccountHash = runtime::get_named_arg("owner");
    let spender: AccountHash = runtime::get_named_arg("spender");
    let value: U256 = runtime::get_named_arg("value");
    let deadline: U256 = runtime::get_named_arg("deadline");
    let v: u8 = runtime::get_named_arg("v");
    let r: [u8; 32] = runtime::get_named_arg("r");
    let s: [u8; 32] = runtime::get_named_arg("s");
    if (deadline < U256::from(&runtime::get_blocktime().to_bytes().unwrap()[..])) {
        runtime::revert(Error::UniswapV2Expired);
    }
    let new_nonce: U256 = get_key::<U256>(&nonce_key(&owner))
    .checked_add(U256::from(1))
    .unwrap_or_revert();
    set_key(&nonce_key(&owner), new_nonce);
    let mut value_bytes = [0u8; 32];
    value.to_big_endian(&mut value_bytes);
    let mut new_nonce_bytes = [0u8; 32];
    new_nonce.to_big_endian(&mut new_nonce_bytes);
    let mut deadline_bytes = [0u8; 32];
    deadline.to_big_endian(&mut deadline_bytes);
    let param = keccak256(&[
        get_key::<[u8; 32]>("permit_typehash").to_vec(),
        owner.value().to_vec(),
        spender.value().to_vec(),
        value_bytes.to_vec(),
        new_nonce_bytes.to_vec(),
        deadline_bytes.to_vec()
    ].concat()[..]);
    let digest = keccak256(&[
        "\x19\x01".as_bytes().to_vec(),
        get_key::<[u8; 32]>("domain_separator").to_vec(),
        //[0u8; 32].to_vec(),
        param.to_vec()
    ].concat()[..]);
    let recoveredAccountHash = ecrecover::ecrecover_sol(&digest, v, r, s);
    if (recoveredAccountHash == AccountHash::new([0u8; 32]) || recoveredAccountHash != owner) {
        runtime::revert(Error::UniswapV2InvalidSignature);
    }
    _approve(owner, spender, value);
}

#[no_mangle]
pub extern "C" fn call() {
    let token_name: String = "Uniswap V2".to_string();
    let token_symbol: String = "UNI-V2".to_string();
    let token_decimals: u8 = 18;
    let token_total_supply: U256 = runtime::get_named_arg("token_total_supply");
    let permit_typehash  = keccak256(b"Permit(owner: AccountHash,spender: AccountHash,value: U256,nonce: U256,deadline: U256)");
    let domain_separator = [0u8; 32];
    // we are setting domain_separator properly using _set_domain_separator-----
    // needs to be done after deploying the contract since we need its hash
    // current problem is that the value we're getting when querying domain_separator in tests remains [0u8; 32]
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(endpoint("name", vec![], CLType::String));
    entry_points.add_entry_point(endpoint("symbol", vec![], CLType::String));
    entry_points.add_entry_point(endpoint("decimals", vec![], CLType::U8));
    entry_points.add_entry_point(endpoint("total_supply", vec![], CLType::U32));
    entry_points.add_entry_point(endpoint("permit_typehash", vec![], CLType::ByteArray(32)));
    entry_points.add_entry_point(endpoint("domain_separator", vec![], CLType::ByteArray(32)));
    entry_points.add_entry_point(endpoint(
        "nonces",
        vec![Parameter::new("account", AccountHash::cl_type())],
        CLType::U256,
    ));
    entry_points.add_entry_point(endpoint(
        "transfer",
        vec![
            Parameter::new("recipient", AccountHash::cl_type()),
            Parameter::new("amount", CLType::U256),
        ],
        CLType::Unit,
    ));
    entry_points.add_entry_point(endpoint(
        "balance_of",
        vec![Parameter::new("account", AccountHash::cl_type())],
        CLType::U256,
    ));
    entry_points.add_entry_point(endpoint(
        "allowance",
        vec![
            Parameter::new("owner", AccountHash::cl_type()),
            Parameter::new("spender", AccountHash::cl_type()),
        ],
        CLType::U256,
    ));
    entry_points.add_entry_point(endpoint(
        "approve",
        vec![
            Parameter::new("spender", AccountHash::cl_type()),
            Parameter::new("amount", CLType::U256),
        ],
        CLType::Unit,
    ));
    entry_points.add_entry_point(endpoint(
        "transfer_from",
        vec![
            Parameter::new("owner", AccountHash::cl_type()),
            Parameter::new("recipient", AccountHash::cl_type()),
            Parameter::new("amount", CLType::U256),
        ],
        CLType::Unit,
    ));
    entry_points.add_entry_point(endpoint(
        "permit",
        vec![
            Parameter::new("owner", AccountHash::cl_type()),
            Parameter::new("spender", AccountHash::cl_type()),
            Parameter::new("value", CLType::U256),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("v", CLType::U8),
            Parameter::new("r", CLType::ByteArray(32)),
            Parameter::new("s", CLType::ByteArray(32)),
        ],
        CLType::Unit,
    ));

    let mut named_keys = NamedKeys::new();
    named_keys.insert("name".to_string(), storage::new_uref(token_name).into());
    named_keys.insert("symbol".to_string(), storage::new_uref(token_symbol).into());
    named_keys.insert(
        "decimals".to_string(),
        storage::new_uref(token_decimals).into(),
    );
    named_keys.insert(
        "total_supply".to_string(),
        storage::new_uref(token_total_supply).into(),
    );
    named_keys.insert(
        "permit_typehash".to_string(),
        storage::new_uref(permit_typehash).into(),
    );
    named_keys.insert(
        "domain_separator".to_string(),
        storage::new_uref(domain_separator).into(),
    );
    named_keys.insert(
        balance_key(&runtime::get_caller()),
        storage::new_uref(token_total_supply).into(),
    );
    named_keys.insert(
        nonce_key(&runtime::get_caller()),
        storage::new_uref(U256::from(0)).into(),
    );

    let (contract_hash, _) =
        storage::new_locked_contract(entry_points, Some(named_keys), None, None);
    runtime::put_key("UNI-V2", contract_hash.into());
    runtime::put_key("UNI_V2_hash", storage::new_uref(contract_hash).into());
    _set_domain_separator(contract_hash);
}

fn _set_domain_separator(hash: ContractHash) {
    let mut contract_address = [0u8; 32];
    contract_address.copy_from_slice(&hash.as_bytes());
    let param = &[
        keccak256(b"EIP712Domain(name: String,version: String,chainId: String,verifyingContract: ContractHash)").to_vec(),
        keccak256(b"Uniswap V2").to_vec(),
        keccak256(b"1").to_vec(),
        "casper-test".as_bytes().to_vec(),
        contract_address.to_vec()
    ].concat()[..];
    let domain_separator = keccak256(param);
    set_key("domain_separator", domain_separator);
}

fn _transfer(sender: AccountHash, recipient: AccountHash, amount: U256) {
    _check_accounts_not_null(sender, recipient);
    let sender_key = balance_key(&sender);
    let recipient_key = balance_key(&recipient);
    let new_sender_balance: U256 = get_key::<U256>(&sender_key)
    .checked_sub(amount)
    .unwrap_or_revert();
    set_key(&sender_key, new_sender_balance);
    let new_recipient_balance: U256 = get_key::<U256>(&recipient_key)
    .checked_add(amount)
    .unwrap_or_revert();
    set_key(&recipient_key, new_recipient_balance);
}

fn _transfer_from(owner: AccountHash, recipient: AccountHash, amount: U256) {
    _check_accounts_not_null(owner, recipient);
    let key = allowance_key(&owner, &runtime::get_caller());
    _transfer(owner, recipient, amount);
    _approve(
        owner,
        runtime::get_caller(),
        get_key::<U256>(&key)
        .checked_sub(amount)
        .unwrap_or_revert(),
    );
}

fn _mint(to: AccountHash, value: U256) {
    let total_supply: U256 = get_key::<U256>("total_supply")
    .checked_add(value)
    .unwrap_or_revert();
    set_key("total_supply", total_supply);
    let to_key = balance_key(&to);
    let new_to_balance: U256 = get_key::<U256>(&to_key)
    .checked_add(value)
    .unwrap_or_revert();
    set_key(&to_key, new_to_balance);
}

fn _burn(from: AccountHash, value: U256) {
    let from_key = balance_key(&from);
    let new_from_balance: U256 = get_key::<U256>(&from_key)
    .checked_sub(value)
    .unwrap_or_revert();
    set_key(&from_key, new_from_balance);
    let total_supply: U256 = get_key::<U256>("total_supply")
    .checked_sub(value)
    .unwrap_or_revert();
    set_key("total_supply", total_supply);
}

fn _approve(owner: AccountHash, spender: AccountHash, amount: U256) {
    _check_accounts_not_null(owner, spender);
    set_key(&allowance_key(&owner, &spender), amount);
}

fn _check_accounts_not_null(x: AccountHash, y:AccountHash) {
    if x == AccountHash::default() || y == AccountHash::default() {
        runtime::revert(Error::UniswapV2ZeroAddress);
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

fn balance_key(account: &AccountHash) -> String {
    format!("balances_{}", account)
}

fn allowance_key(owner: &AccountHash, sender: &AccountHash) -> String {
    format!("allowances_{}_{}", owner, sender)
}

fn nonce_key(account: &AccountHash) -> String {
    format!("nonces_{}", account)
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