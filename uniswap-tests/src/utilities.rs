use casper_types::{ContractHash, U256, account::AccountHash};
use renvm_sig::keccak256;
use crate::uniswap_erc20::Token as UNI_Token;

pub fn get_domain_separator(hash: ContractHash) -> [u8; 32] {
        let param = &[
            keccak256(b"EIP712Domain(name: String,version: String,chainId: String,verifyingContract: ContractHash)").to_vec(),
            keccak256(b"Uniswap V2").to_vec(),
            keccak256(b"1").to_vec(),
            "casper-test".as_bytes().to_vec(),
            hash.value().to_vec()
        ].concat()[..];
        keccak256(param)
        //assert_eq!(hash, [150, 237, 213, 190, 234, 171, 113, 245, 77, 1, 58, 26, 239, 79, 74, 196, 184, 236, 187, 23, 187, 64, 50, 45, 169, 230, 122, 37, 192, 165, 27, 62]);
}

pub fn get_approval_digest(
    token: &UNI_Token,
    owner: AccountHash,
    spender: AccountHash,
    value: U256,
    nonce: U256,
    deadline: U256
) -> [u8; 32] {
    //let domain_separator = get_domain_separator(ContractHash::from(token.contract_hash()));
    let mut value_bytes = [0u8; 32];
    value.to_big_endian(&mut value_bytes);
    let mut nonce_bytes = [0u8; 32];
    nonce.to_big_endian(&mut nonce_bytes);
    let mut deadline_bytes = [0u8; 32];
    deadline.to_big_endian(&mut deadline_bytes);
    let param = keccak256(&[
        token.permit_typehash().to_vec(),
        owner.value().to_vec(),
        spender.value().to_vec(),
        value_bytes.to_vec(),
        nonce_bytes.to_vec(),
        deadline_bytes.to_vec()
    ].concat()[..]);
    keccak256(&[
        "\x19\x01".as_bytes().to_vec(),
        //get_domain_separator(token.contract_hash())
        [0u8; 32].to_vec(),
        param.to_vec()
    ].concat()[..])
}