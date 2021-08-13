use casper_engine_test_support::{Code, Hash, SessionBuilder, TestContext, TestContextBuilder};
use casper_types::{AsymmetricType, CLTyped, PublicKey, RuntimeArgs, U256, U512, account::AccountHash, bytesrepr::FromBytes, runtime_args};
use libsecp256k1::{SecretKey};
//use uniswap_libs::converters::set_size_32;
//use renvm_sig::keccak256;

// contains methods that can simulate a real-world deployment (storing the contract in the blockchain)
// and transactions to invoke the methods in the contract.

pub mod token_cfg {
    use super::*;
    pub const NAME: &str = "Uniswap V2";
    pub const SYMBOL: &str = "UNI-V2";
    pub const DECIMALS: u8 = 18;
    pub const PERMIT_TYPEHASH: [u8; 32] = [
        196, 145, 155, 88, 31, 120, 34, 95, 56, 141, 115, 176,10, 228, 33, 29,
        229, 113, 196, 78, 79, 126, 214, 110, 157, 225, 57, 117, 72, 198, 55, 2
    ];
    pub const DOMAIN_SEPARATOR: [u8; 32] = [
        34, 166, 68, 188, 180, 157, 190, 242, 151, 250, 248, 18, 112, 111, 182,
        149, 89, 117, 155, 63, 64, 214, 166, 171, 253, 34, 213, 211, 7, 31, 75, 245
    ];
    pub fn total_supply() -> U256 {
        1_000.into()
    }
}

pub struct Sender(pub AccountHash);

pub struct Token {
    context: TestContext,
    pub ali_sec: SecretKey,
    pub ali: AccountHash,
    pub bob: AccountHash,
    pub joe: AccountHash,
}

impl Token {
    pub fn deployed() -> Token {
        let ali_seckey = SecretKey::parse(&[3u8; 32]).unwrap();
        let pubkey = libsecp256k1::PublicKey::from_secret_key(&ali_seckey);
        let ali = PublicKey::secp256k1_from_bytes(pubkey.serialize()).unwrap();
        //let ali = PublicKey::ed25519_from_bytes(set_size_32(&pubkey.serialize()[..])).unwrap();
        //let ali = PublicKey::ed25519_from_bytes([3u8; 32]).unwrap();
        let bob = PublicKey::ed25519_from_bytes([6u8; 32]).unwrap();
        let joe = PublicKey::ed25519_from_bytes([9u8; 32]).unwrap();

        let mut context = TestContextBuilder::new()
            .with_public_key(ali.clone(), U512::from(500_000_000_000_000_000u64))
            .with_public_key(bob.clone(), U512::from(500_000_000_000_000_000u64))
            .build();
        let session_code = Code::from("uniswap_erc20.wasm");
        let session_args = runtime_args! {
            "token_total_supply" => token_cfg::total_supply()
        };
        let session = SessionBuilder::new(session_code, session_args)
            .with_address((&ali).to_account_hash())
            .with_authorization_keys(&[ali.to_account_hash()])
            .build();
        context.run(session);
        Token {
            context,
            ali_sec: ali_seckey,
            ali: ali.to_account_hash(),
            bob: bob.to_account_hash(),
            joe: joe.to_account_hash(),
        }
    }

    pub fn contract_hash(&self) -> Hash {
        self.context
            .query(self.ali, &[format!("{}_hash", "UNI_V2")])
            .unwrap_or_else(|_| panic!("{} contract not found", token_cfg::NAME))
            .into_t()
            .unwrap_or_else(|_| panic!("{} has wrong type", token_cfg::NAME))
    }

    /// query a contract's named key.
    fn query_contract<T: CLTyped + FromBytes>(&self, name: &str) -> Option<T> {
        match self
            .context
            .query(self.ali, &[token_cfg::SYMBOL.to_string(), name.to_string()])
        {
            Err(_) => None,
            Ok(maybe_value) => {
                let value = maybe_value
                    .into_t()
                    .unwrap_or_else(|_| panic!("{} is not the expected type.", name));
                Some(value)
            }
        }
    }

    /// call a contract's specific entry point.
    fn call(&mut self, sender: Sender, method: &str, args: RuntimeArgs) {
        let Sender(address) = sender;
        let code = Code::Hash(self.contract_hash(), method.to_string());
        let session = SessionBuilder::new(code, args)
            .with_address(address)
            .with_authorization_keys(&[address])
            .build();
        self.context.run(session);
    }

    pub fn name(&self) -> String {
        self.query_contract("name").unwrap()
    }

    pub fn symbol(&self) -> String {
        self.query_contract("symbol").unwrap()
    }

    pub fn decimals(&self) -> u8 {
        self.query_contract("decimals").unwrap()
    }

    pub fn total_supply(&self) -> U256 {
        self.query_contract("total_supply").unwrap()
    }

    pub fn permit_typehash(&self) -> [u8; 32] {
        self.query_contract("permit_typehash").unwrap()
    }

    pub fn domain_separator(&self) -> [u8; 32] {
        self.query_contract("domain_separator").unwrap()
    }

    pub fn nonces(&self, account: AccountHash) -> U256 {
        let key = format!("nonces_{}", account);
        self.query_contract(&key).unwrap_or_default()
    }

    pub fn balance_of(&self, account: AccountHash) -> U256 {
        let key = format!("balances_{}", account);
        self.query_contract(&key).unwrap_or_default()
    }

    pub fn allowance(&self, owner: AccountHash, spender: AccountHash) -> U256 {
        let key = format!("allowances_{}_{}", owner, spender);
        self.query_contract(&key).unwrap_or_default()
    }

    pub fn transfer(&mut self, recipient: AccountHash, amount: U256, sender: Sender) {
        self.call(
            sender,
            "transfer",
            runtime_args! {
                "recipient" => recipient,
                "amount" => amount
            },
        );
    }

    pub fn approve(&mut self, spender: AccountHash, amount: U256, sender: Sender) {
        self.call(
            sender,
            "approve",
            runtime_args! {
                "spender" => spender,
                "amount" => amount
            },
        );
    }

    pub fn transfer_from(
        &mut self,
        owner: AccountHash,
        recipient: AccountHash,
        amount: U256,
        sender: Sender,
    ) {
        self.call(
            sender,
            "transfer_from",
            runtime_args! {
                "owner" => owner,
                "recipient" => recipient,
                "amount" => amount
            },
        );
    }

    pub fn permit(
        &mut self,
        owner: AccountHash,
        spender: AccountHash,
        value: U256,
        deadline: U256,
        v: u8,
        r: [u8; 32],
        s: [u8; 32],
        sender: Sender,
    ) {
        self.call(
            sender,
            "permit",
            runtime_args! {
                "owner" => owner,
                "spender" => spender,
                "value" => value,
                "deadline" => deadline,
                "v" => v,
                "r" => r,
                "s" => s
            },
        );
    }
}