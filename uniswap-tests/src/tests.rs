use casper_types::U256;
use libsecp256k1::{Message, sign};
use libsecp256k1::curve::Scalar;
use uniswap_libs::{converters::{u8_32_to_u32_8, set_size_32}, ecrecover::ecrecover_sol};

use crate::erc20::{token_cfg, Sender, Token};
use crate::uniswap_erc20::{token_cfg as UNI_token_cfg, Sender as UNI_Sender, Token as UNI_Token};
use crate::utilities::get_approval_digest;

// ------------ START - ERC20 Tests ------------

#[test]
fn test_erc20_deploy() {
    let t = Token::deployed();
    assert_eq!(t.name(), token_cfg::NAME);
    assert_eq!(t.symbol(), token_cfg::SYMBOL);
    assert_eq!(t.decimals(), token_cfg::DECIMALS);
    assert_eq!(t.balance_of(t.ali), token_cfg::total_supply());
    assert_eq!(t.balance_of(t.bob), 0.into());
    assert_eq!(t.allowance(t.ali, t.ali), 0.into());
    assert_eq!(t.allowance(t.ali, t.bob), 0.into());
    assert_eq!(t.allowance(t.bob, t.ali), 0.into());
    assert_eq!(t.allowance(t.bob, t.bob), 0.into());
}

#[test]
fn test_erc20_transfer() {
    let amount = 10.into();
    let mut t = Token::deployed();
    t.transfer(t.bob, amount, Sender(t.ali));
    assert_eq!(t.balance_of(t.ali), token_cfg::total_supply() - amount);
    assert_eq!(t.balance_of(t.bob), amount);
}

#[test]
#[should_panic]
fn test_erc20_transfer_too_much() {
    let amount = 1.into();
    let mut t = Token::deployed();
    t.transfer(t.ali, amount, Sender(t.bob));
}

#[test]
fn test_erc20_approve() {
    let amount = 10.into();
    let mut t = Token::deployed();
    t.approve(t.bob, amount, Sender(t.ali));
    assert_eq!(t.balance_of(t.ali), token_cfg::total_supply());
    assert_eq!(t.balance_of(t.bob), 0.into());
    assert_eq!(t.allowance(t.ali, t.bob), amount);
    assert_eq!(t.allowance(t.bob, t.ali), 0.into());
}

#[test]
fn test_erc20_transfer_from() {
    let allowance = 10.into();
    let amount = 3.into();
    let mut t = Token::deployed();
    t.approve(t.bob, allowance, Sender(t.ali));
    t.transfer_from(t.ali, t.joe, amount, Sender(t.bob));
    assert_eq!(t.balance_of(t.ali), token_cfg::total_supply() - amount);
    assert_eq!(t.balance_of(t.bob), 0.into());
    assert_eq!(t.balance_of(t.joe), amount);
    assert_eq!(t.allowance(t.ali, t.bob), allowance - amount);
}

#[test]
#[should_panic]
fn test_erc20_transfer_from_too_much() {
    let amount = token_cfg::total_supply().checked_add(1.into()).unwrap();
    let mut t = Token::deployed();
    t.transfer_from(t.ali, t.joe, amount, Sender(t.bob));
}

// ------------ START - UNISWAP_ERC20 Tests ------------

#[test]
fn test_uniswap_erc20_deploy() {
    let t = UNI_Token::deployed();
    assert_eq!(t.name(), UNI_token_cfg::NAME);
    assert_eq!(t.symbol(), UNI_token_cfg::SYMBOL);
    assert_eq!(t.decimals(), UNI_token_cfg::DECIMALS);
    assert_eq!(t.balance_of(t.ali), UNI_token_cfg::total_supply());
    assert_eq!(t.balance_of(t.bob), 0.into());
    assert_eq!(t.allowance(t.ali, t.ali), 0.into());
    assert_eq!(t.allowance(t.ali, t.bob), 0.into());
    assert_eq!(t.allowance(t.bob, t.ali), 0.into());
    assert_eq!(t.allowance(t.bob, t.bob), 0.into());
    assert_eq!(t.nonces(t.ali), 0.into());
    assert_eq!(t.permit_typehash(), UNI_token_cfg::PERMIT_TYPEHASH);
    assert_eq!(t.domain_separator(), [0u8; 32]);
}

#[test]
fn test_uniswap_erc20_transfer() {
    let amount = 10.into();
    let mut t = UNI_Token::deployed();
    t.transfer(t.bob, amount, UNI_Sender(t.ali));
    assert_eq!(t.balance_of(t.ali), UNI_token_cfg::total_supply() - amount);
    assert_eq!(t.balance_of(t.bob), amount);
}

#[test]
#[should_panic]
fn test_uniswap_erc20_transfer_too_much() {
    let amount = 1.into();
    let mut t = UNI_Token::deployed();
    t.transfer(t.ali, amount, UNI_Sender(t.bob));
}

#[test]
fn test_uniswap_erc20_approve() {
    let amount = 10.into();
    let mut t = UNI_Token::deployed();
    t.approve(t.bob, amount, UNI_Sender(t.ali));
    assert_eq!(t.balance_of(t.ali), UNI_token_cfg::total_supply());
    assert_eq!(t.balance_of(t.bob), 0.into());
    assert_eq!(t.allowance(t.ali, t.bob), amount);
    assert_eq!(t.allowance(t.bob, t.ali), 0.into());
}

#[test]
fn test_uniswap_erc20_transfer_from() {
    let allowance = 10.into();
    let amount = 3.into();
    let mut t = UNI_Token::deployed();
    t.approve(t.bob, allowance, UNI_Sender(t.ali));
    t.transfer_from(t.ali, t.joe, amount, UNI_Sender(t.bob));
    assert_eq!(t.balance_of(t.ali), UNI_token_cfg::total_supply() - amount);
    assert_eq!(t.balance_of(t.bob), 0.into());
    assert_eq!(t.balance_of(t.joe), amount);
    assert_eq!(t.allowance(t.ali, t.bob), allowance - amount);
}

#[test]
#[should_panic]
fn test_uniswap_erc20_transfer_from_too_much() {
    let amount = UNI_token_cfg::total_supply().checked_add(1.into()).unwrap();
    let mut t = UNI_Token::deployed();
    t.transfer_from(t.ali, t.joe, amount, UNI_Sender(t.bob));
}

#[test]
fn test_uniswap_erc20_permit() {
    let mut t = UNI_Token::deployed();
    let value = U256::from(10*18);
    let nonce = t.nonces(t.ali);
    let deadline = U256::MAX;
    let digest = get_approval_digest(&t, t.ali, t.joe, value, nonce + U256::from(1), deadline);
    //assert_eq!(digest, [0u8; 32]);
    let msg = Message(Scalar(u8_32_to_u32_8(digest)));
    let (signature, v) = sign(&msg, &(t.ali_sec));
    let sig: [u8; 64] = signature.serialize();
    let (r, s) = sig.split_at(32);
    let output = ecrecover_sol(&digest, v.serialize(), set_size_32(r), set_size_32(s));
    assert_eq!(t.ali, output);
    t.permit(
        t.ali, 
        t.joe, 
        value, 
        deadline, 
        v.serialize(), 
        set_size_32(r), 
        set_size_32(s), 
        UNI_Sender(t.ali)
    );
    assert_eq!(t.allowance(t.ali, t.joe), value);
    assert_eq!(t.nonces(t.ali), nonce + U256::from(1));
}