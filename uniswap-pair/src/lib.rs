#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};
use uniswap_erc20::balance_of;
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
    UniswapV2InsufficientLiquidityMinted = 5,
    UniswapV2InsufficientLiquidityBurned = 6,
    UniswapV2Locked = 7,
    UniswapV2InsufficientInputAmount = 8,
    UniswapV2InsufficientOutputAmount = 9,
    UniswapV2InsufficientLiquidity = 10,
    UniswapV2InvalidTo = 11,
    UniswapV2K = 12
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
// START - uniswap-erc20 getters
#[no_mangle]
pub extern "C" fn name() {
    let val: String = get_key("name");
    ret(val)
}

#[no_mangle]
pub extern "C" fn symbol() {
    let val: String = get_key("symbol");
    ret(val)
}

#[no_mangle]
pub extern "C" fn decimals() {
    let val: u8 = get_key("decimals");
    ret(val)
}

#[no_mangle]
pub extern "C" fn total_supply() {
    let val: U256 = get_key("total_supply");
    ret(val)
}

#[no_mangle]
pub extern "C" fn permit_typehash() {
    let val: [u8; 32] = get_key("permit_typehash");
    ret(val)
}
// END - uniswap-erc20 getters
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
    ret(result)
}
// ***************** GETTERS - END ********************

/// Function: mint(to: ContractHash) -> liquidity: U256
///
/// # Purpose
/// creates pool tokens.
/// # Arguments
/// * `to` - An ContractHash that holds the pool's contract hash.
/// # Returns
/// * `liquidity` - The newly created pool liquidity.
///
/// this low-level function should be called from a contract which performs important safety checks.
#[no_mangle]
extern "C" fn mint() {
    // alternative for the lock() access modifier in Solidity
    // serves as a mutex for external functions to mitigate reentrancy attacks
    if (get_key::<U256>("unlocked") != U256::from(1)) {
        runtime::revert(Error::UniswapV2Locked);
    }
    set_key("unlocked", U256::from(0));
    let to: ContractHash = runtime::get_named_arg("to");
    let _reserve0: [u8; 14] = get_key::<[u8; 14]>("reserve0");
    let _reserve1: [u8; 14] = get_key::<[u8; 14]>("reserve1");
    let current_contract_hash: ContractHash = get_key::<ContractHash>(&get_key::<String>("name"));
    let mut named_args = RuntimeArgs::new();
    match AccountHash::from_bytes(current_contract_hash.as_bytes()) {
        Ok(hash) => {
            named_args.insert("account", hash.0).unwrap();
        }
        Err(e) => eprintln!("Error @pair::mint - {}", e)
    }
    let balance0: U256 = call_contract(
        get_key::<ContractHash>("token0"),
        "balance_of",
        named_args.clone());
    let balance1: U256 = call_contract(
        get_key::<ContractHash>("token1"),
        "balance_of",
        named_args.clone());
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
                    let fee_on: bool = _mintFee(Uint112(_reserve0), Uint112(_reserve1));
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
                        // free the lock
                        set_key("unlocked", U256::from(1));
                        runtime::revert(Error::UniswapV2InsufficientLiquidityMinted);
                    }
                    match AccountHash::from_bytes(to.as_bytes()) {
                        Ok(res) => {
                            _mint(res.0, liquidity);
                            _update(balance0, balance1, Uint112(_reserve0), Uint112(_reserve1));
                            if (fee_on) {
                                set_key::<U256>("kLast", res_0.0.mul(res_1.0));
                            }
                            // free the lock
                            set_key("unlocked", U256::from(1));
                            ret(liquidity);
                        },
                        Err(e) => eprintln!("Error @pair::mint - {}", e)
                    }
                }
                Err(e) => eprintln!("Error @pair::mint - {}", e)
            }
        }
        Err(e) => eprintln!("Error @pair::mint - {}", e)
    }
    // free the lock
    set_key("unlocked", U256::from(1));
}

/// Function: burn(to: AccountHash) -> (amount0: U256, amount1: U256)
///
/// # Purpose
/// destroys pool tokens.
/// # Arguments
/// * `to` - An AccountHash that holds the liquidity provider's account hash.
/// # Returns
/// * `amount0` - the first token's amount burned from the pool and transfered to the provider.
/// * `amount1` - the second token's amount burned from the pool and transfered to the provider.
///
/// this low-level function should be called from a contract which performs important safety checks.
#[no_mangle]
extern "C" fn burn() {
    // alternative for the lock() access modifier in Solidity
    // serves as a mutex for external functions to mitigate reentrancy attacks
    if (get_key::<U256>("unlocked") != U256::from(1)) {
        runtime::revert(Error::UniswapV2Locked);
    }
    set_key("unlocked", U256::from(0));
    let to: AccountHash = runtime::get_named_arg("to");
    // getting the pair's tokens' reserves & hashes
    let _reserve0: [u8; 14] = get_key::<[u8; 14]>("reserve0");
    let _reserve1: [u8; 14] = get_key::<[u8; 14]>("reserve1");
    let _token0: ContractHash = get_key::<ContractHash>("token0");
    let _token1: ContractHash = get_key::<ContractHash>("token1");
    // get current contract hash
    let current_contract_hash: ContractHash = get_key::<ContractHash>(&get_key::<String>("name"));
    let mut named_args = RuntimeArgs::new();
    match AccountHash::from_bytes(current_contract_hash.as_bytes()) {
        Ok(hash) => {
            named_args.insert("account", hash.0).unwrap();
        }
        Err(e) => eprintln!("Error @pair::burn - {}", e)
    }
    // getting the tokens' balances for the current pair
    let mut balance0: U256 = call_contract(_token0, "balance_of", named_args.clone());
    let mut balance1: U256 = call_contract(_token1, "balance_of", named_args.clone());
    let liquidity: U256 = call_contract(
        get_key::<ContractHash>("UNI-V2"),
        "balance_of",
        named_args.clone());
    let fee_on: bool = _mintFee(Uint112(_reserve0), Uint112(_reserve1));
    let _total_supply: U256 = get_key::<U256>("total_supply");
    // return params are amount0 and amount1
    let amount0: U256 = (liquidity.mul(balance0)).div(_total_supply);
    let amount1: U256 = (liquidity.mul(balance1)).div(_total_supply);
    if (amount0 <= U256::from(0) || amount1 <= U256::from(0)) {
        // free the lock
        set_key("unlocked", U256::from(1));
        runtime::revert(Error::UniswapV2InsufficientLiquidityBurned);
    }
    //_burn(address(this), liquidity);
    _safeTransfer(_token0, to, amount0);
    _safeTransfer(_token1, to, amount1);
    // get latest balances after the transfers
    balance0 = call_contract(_token0, "balance_of", named_args.clone());
    balance1 = call_contract(_token1, "balance_of", named_args.clone());
    // update the pool's reserves
    _update(balance0, balance1, Uint112(_reserve0), Uint112(_reserve1));
    if (fee_on) {
        match U256::from_bytes(&Uint112(_reserve0).encode()[..]) {
            Ok(res_0) => {
                match U256::from_bytes(&Uint112(_reserve1).encode()[..]) {
                    Ok(res_1) => {
                        set_key::<U256>("kLast", res_0.0.mul(res_1.0));
                    }
                    Err(e) => eprintln!("Error @pair::burn - {}", e)
                }
            }
            Err(e) => eprintln!("Error @pair::burn - {}", e)
        }
    }
    // free the lock
    set_key("unlocked", U256::from(1));
    ret((amount0, amount1))
}

/// Function: swap(amount0_out: U256, amount1_out: U256, to: AccountHash, data: Bytes)
///
/// # Purpose
/// swaps tokens. For regular swaps, data.length must be 0. Also see Flash Swaps.
/// # Arguments
/// * `amount0_out` - the first token's amount transfered from the pool to the provider.
/// * `amount1_out` - the second token's amount transfered from the pool to the provider.
/// * `to` - An AccountHash that holds the liquidity provider's account hash.
/// * `data` - A Bytes that indicates the nature of the swap (regular or flash).
///
/// this low-level function should be called from a contract which performs important safety checks.
#[no_mangle]
extern "C" fn swap() {
    // alternative for the lock() access modifier in Solidity
    // serves as a mutex for external functions to mitigate reentrancy attacks
    if (get_key::<U256>("unlocked") != U256::from(1)) {
        runtime::revert(Error::UniswapV2Locked);
    }
    set_key("unlocked", U256::from(0));
    let amount0_out: U256 = runtime::get_named_arg("amount0_out");
    let amount1_out: U256 = runtime::get_named_arg("amount1_out");
    let to: AccountHash = runtime::get_named_arg("to");
    //let data: Bytes = runtime::get_named_arg("data");
    if (amount0_out <= U256::from(0) && amount1_out <= U256::from(0)) {
        // free the lock
        set_key("unlocked", U256::from(1));
        runtime::revert(Error::UniswapV2InsufficientOutputAmount);
    }
    // getting the pair's tokens' reserves
    let _reserve0: [u8; 14] = get_key::<[u8; 14]>("reserve0");
    let _reserve1: [u8; 14] = get_key::<[u8; 14]>("reserve1");
    match U256::from_bytes(&Uint112(_reserve0).encode()[..]) {
        Ok(res_O) => {
            match U256::from_bytes(&Uint112(_reserve1).encode()[..]) {
                Ok(res_1) => {
                    if (amount0_out >= res_O.0 || amount1_out >= res_1.0) {
                        // free the lock
                        set_key("unlocked", U256::from(1));
                        runtime::revert(Error::UniswapV2InsufficientLiquidity);
                    }
                    let balance0: U256;
                    let balance1: U256;
                    { // scope for _token{0,1}, avoids stack too deep errors
                        let _token0: ContractHash = get_key::<ContractHash>("token0");
                        let _token1: ContractHash = get_key::<ContractHash>("token1");
                        if (to.value() == _token0.value() || to.value() == _token1.value()) {
                            // free the lock
                            set_key("unlocked", U256::from(1));
                            runtime::revert(Error::UniswapV2InvalidTo);
                        }
                        if (amount0_out > U256::from(0)) {
                            _safeTransfer(_token0, to, amount0_out); // optimistically transfer tokens
                        }
                        if (amount1_out > U256::from(0)) {
                            _safeTransfer(_token1, to, amount1_out); // optimistically transfer tokens
                        }
                        // ---call to uniswapV2Call fct which is not implemented in the original UniswapV2---
                        // get current contract hash
                        let current_contract_hash: ContractHash = get_key::<ContractHash>(&get_key::<String>("name"));
                        let mut named_args = RuntimeArgs::new();
                        match AccountHash::from_bytes(current_contract_hash.as_bytes()) {
                            Ok(hash) => {
                                named_args.insert("account", hash.0).unwrap();
                            }
                            Err(e) => eprintln!("Error @pair::swap - {}", e)
                        }
                        // get latest balances after the transfers
                        balance0 = call_contract(_token0, "balance_of", named_args.clone());
                        balance1 = call_contract(_token1, "balance_of", named_args.clone());
                    }
                    let amountO_in: U256 = if (balance0 > res_O.0 - amount0_out) {
                        balance0 - (res_O.0 - amount0_out)
                    } else {
                        U256::from(0)
                    };
                    let amount1_in: U256 = if (balance1 > res_1.0 - amount1_out) {
                        balance1 - (res_1.0 - amount1_out)
                    } else {
                        U256::from(0)
                    };
                    if (amountO_in <= U256::from(0) && amount1_in <= U256::from(0)) {
                        // free the lock
                        set_key("unlocked", U256::from(1));
                        runtime::revert(Error::UniswapV2InsufficientInputAmount);
                    }
                    { // scope for reserve{0,1}Adjusted, avoids stack too deep errors
                        let balance0_adjusted: U256 = balance0.mul(U256::from(1000)).sub(amountO_in.mul(U256::from(3)));
                        let balance1_adjusted: U256 = balance1.mul(U256::from(1000)).sub(amount1_in.mul(U256::from(3)));
                        if (balance0_adjusted.mul(balance1_adjusted) < res_O.0.mul(res_1.0).mul(U256::from(1000i32.pow(2)))) {
                            // free the lock
                            set_key("unlocked", U256::from(1));
                            runtime::revert(Error::UniswapV2K);
                        }
                    }
                    _update(balance0, balance1, Uint112(_reserve0), Uint112(_reserve1));
                }
                Err(e) => eprintln!("Error @pair::swap - {}", e)
            }
        }
        Err(e) => eprintln!("Error @pair::swap - {}", e)
    }
    // free the lock
    set_key("unlocked", U256::from(1));
}

/// Function: skim(to: ContractHash)
///
/// # Purpose
/// forces balances to match reserves.
/// # Arguments
/// * `to` - An ContractHash that holds the pool's contract hash.
#[no_mangle]
extern "C" fn skim() {
    // alternative for the lock() access modifier in Solidity
    // serves as a mutex for external functions to mitigate reentrancy attacks
    if (get_key::<U256>("unlocked") != U256::from(1)) {
        runtime::revert(Error::UniswapV2Locked);
    }
    set_key("unlocked", U256::from(0));
    let to: AccountHash = runtime::get_named_arg("to");
    let _token0: ContractHash = get_key::<ContractHash>("token0");
    let _token1: ContractHash = get_key::<ContractHash>("token1");
    // get current contract hash
    let current_contract_hash: ContractHash = get_key::<ContractHash>(&get_key::<String>("name"));
    let mut named_args = RuntimeArgs::new();
    match AccountHash::from_bytes(current_contract_hash.as_bytes()) {
        Ok(hash) => {
            named_args.insert("account", hash.0).unwrap();
        }
        Err(e) => eprintln!("Error @pair::skim - {}", e)
    }
    let balance0: U256 = call_contract(_token0, "balance_of", named_args.clone());
    let balance1: U256 = call_contract(_token1, "balance_of", named_args.clone());
    // getting the pair's tokens' reserves
    let _reserve0: [u8; 14] = get_key::<[u8; 14]>("reserve0");
    let _reserve1: [u8; 14] = get_key::<[u8; 14]>("reserve1");
    match U256::from_bytes(&Uint112(_reserve0).encode()[..]) {
        Ok(res0) => {
            match U256::from_bytes(&Uint112(_reserve1).encode()[..]) {
                Ok(res1) => {
                    _safeTransfer(_token0, to, balance0.sub(res0.0));
                    _safeTransfer(_token1, to, balance1.sub(res1.0));
                }
                Err(e) => eprintln!("Error @pair::skim - {}", e)
            }
        }
        Err(e) => eprintln!("Error @pair::skim - {}", e)
    }
    // free the lock
    set_key("unlocked", U256::from(1));
}

/// Function: sync()
///
/// # Purpose 
/// forces reserves to match balances.
#[no_mangle]
extern "C" fn sync() {
    // alternative for the lock() access modifier in Solidity
    // serves as a mutex for external functions to mitigate reentrancy attacks
    if (get_key::<U256>("unlocked") != U256::from(1)) {
        runtime::revert(Error::UniswapV2Locked);
    }
    set_key("unlocked", U256::from(0));
    let _token0: ContractHash = get_key::<ContractHash>("token0");
    let _token1: ContractHash = get_key::<ContractHash>("token1");
    // get current contract hash
    let current_contract_hash: ContractHash = get_key::<ContractHash>(&get_key::<String>("name"));
    let mut named_args = RuntimeArgs::new();
    match AccountHash::from_bytes(current_contract_hash.as_bytes()) {
        Ok(hash) => {
            named_args.insert("account", hash.0).unwrap();
        }
        Err(e) => eprintln!("Error @pair::mint - {}", e)
    }
    let balance0: U256 = call_contract(_token0, "balance_of", named_args.clone());
    let balance1: U256 = call_contract(_token1, "balance_of", named_args.clone());
    // getting the pair's tokens' reserves
    let _reserve0: [u8; 14] = get_key::<[u8; 14]>("reserve0");
    let _reserve1: [u8; 14] = get_key::<[u8; 14]>("reserve1");
    _update(balance0, balance1, Uint112(_reserve0), Uint112(_reserve1));
    // free the lock
    set_key("unlocked", U256::from(1));
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

// update reserves and, on the first call per block, price accumulators
fn _update(balance0: U256, balance1: U256, _reserve0: Uint112, _reserve1: Uint112) {
    if (balance0 > U256::from(2i32.pow(112) - 1) || balance1 > U256::from(2i32.pow(112) - 1)) {
        // free the lock
        set_key("unlocked", U256::from(1));
        runtime::revert(Error::UniswapV2Overflow);
    }
    // assign a value to blockTimestamp just to avoid the error "use of possibly-uninitialized variable"
    let mut blockTimestamp = u32::MAX;
    // Here, we are sure that checked_rem() will result in Some() and not None
    match u64::from(get_blocktime()).checked_rem(2i32.pow(32) as u64) {
        Some(res) => blockTimestamp = res as u32,
        None => eprintln!("Cannot divide by zero @pair::_update")
    }
    let timeElapsed: u32 = blockTimestamp - get_key::<u32>("blockTimestampLast");
    if (timeElapsed > u32::MIN && u128::from_be_bytes(*pop_u128(&(_reserve0.encode())[..])) != u128::MIN && u128::from_be_bytes(*pop_u128(&(_reserve1.encode())[..])) != u128::MIN) {
        let mut price0CumulativeLast: U256 = get_key::<U256>("price0CumulativeLast");
        match U256::from_bytes(&(uq112x112::uqdiv(&uq112x112::encode(&_reserve1), &_reserve0)).encode()[..]) {
            Ok(res) => price0CumulativeLast += res.0,
            Err(e) => eprintln!("Error @pair::_update - {}", e)
        }
        set_key::<U256>("price0CumulativeLast", price0CumulativeLast);
        let mut price1CumulativeLast: U256 = get_key::<U256>("price1CumulativeLast");
        match U256::from_bytes(&(uq112x112::uqdiv(&uq112x112::encode(&_reserve0), &_reserve1)).encode()[..]) {
            Ok(res) => price1CumulativeLast += res.0,
            Err(e) => eprintln!("Error @pair::_update - {}", e)
        }
        set_key::<U256>("price1CumulativeLast", price1CumulativeLast);
    }
    set_key::<[u8; 14]>("reserve0", *pop_u112(&(balance0.as_u128().encode())[..]));
    set_key::<[u8; 14]>("reserve1", *pop_u112(&(balance1.as_u128().encode())[..]));
    set_key::<u32>("blockTimestampLast", blockTimestamp);
}

// if fee is on, mint liquidity equivalent to 1/6th of the growth in sqrt(k)
fn _mintFee(_reserve0: Uint112, _reserve1: Uint112) -> bool {
    let feeTo: AccountHash = call_contract(get_key("factory"), "feeTo", RuntimeArgs::new());
    let fee_on: bool = feeTo.value() != [0u8; 32];
    let _kLast: U256 = get_key("kLast");
    if (fee_on) {
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
                                        if (liquidity > U256::from(0)) {
                                            _mint(feeTo, liquidity)
                                        }
                                    }
                                }
                                None => eprintln!("Multiplication using 'checked_mul' failed due to Overflow")
                            }
                        }
                        Err(e) => eprintln!("Error @pair::_mintFee - {}", e)
                    }
                },
                Err(e) => eprintln!("Error @pair::_mintFee - {}", e)
            }
        }
    }
    else if (_kLast != U256::from(0)) {
        set_key::<U256>("kLast", U256::from(0));
    }
    return fee_on;
}

// we have no access to the uniswap-erc20 contract's internal functions, so we'll define the ones we need here:
// ***** START: unsiwap-erc20 Internal functions *****
fn _mint(to: AccountHash, value: U256) {
    let total_supply: U256 = get_key::<U256>("total_supply").add(value);
    set_key("total_supply", total_supply);
    let to_key = balance_key(&to);
    let new_to_balance: U256 = (get_key::<U256>(&to_key) + value);
    set_key(&to_key, new_to_balance);
}

fn _burn(from: AccountHash, value: U256) {
    let from_key = balance_key(&from);
    let new_from_balance: U256 = (get_key::<U256>(&from_key) - value);
    set_key(&from_key, new_from_balance);
    let total_supply: U256 = get_key::<U256>("total_supply").sub(value);
    set_key("total_supply", total_supply);
}
// ***** END: unsiwap-erc20 Internal functions *****

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

// helper function to convert &[u8] to &[u8; 16]
fn pop_u128(barry: &[u8]) -> &[u8; 16] {
    barry.try_into().expect("slice with incorrect length")
}

// helper function to convert &[u8] to &[u8; 14]
fn pop_u112(barry: &[u8]) -> &[u8; 14] {
    barry.try_into().expect("slice with incorrect length")
}

// ***** START: unsiwap-erc20 keys' getters *****
fn balance_key(account: &AccountHash) -> String {
    format!("balances_{}", account)
}
// ***** END: unsiwap-erc20 keys' getters *****