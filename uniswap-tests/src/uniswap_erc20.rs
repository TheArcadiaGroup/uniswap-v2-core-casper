use casper_engine_test_support::{Code, Hash, SessionBuilder, TestContext, TestContextBuilder};
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, runtime_args, AsymmetricType, CLTyped, PublicKey,
    RuntimeArgs, U256, U512,
};

// contains methods that can simulate a real-world deployment (storing the contract in the blockchain)
// and transactions to invoke the methods in the contract.

pub mod token_cfg {
    use super::*;
    pub const NAME: &str = "Uniswap V2";
    pub const SYMBOL: &str = "UNI-V2";
    pub const DECIMALS: u8 = 18;
    pub fn total_supply() -> U256 {
        1_000.into()
    }
}

pub struct Sender(pub AccountHash);

pub struct Token {
    context: TestContext,
    pub ali: AccountHash,
    pub bob: AccountHash,
    pub joe: AccountHash,
}

impl Token {
    pub fn deployed() -> Token {   
        let ali = PublicKey::ed25519_from_bytes([3u8; 32]).unwrap();
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
            ali: ali.to_account_hash(),
            bob: bob.to_account_hash(),
            joe: joe.to_account_hash(),
        }
    }

    fn contract_hash(&self) -> Hash {
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
                    .unwrap_or_else(|_| panic!("{} is not expected type.", name));
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