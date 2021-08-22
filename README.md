# Casper Implementation of uniswap-v2-core
Uniswap V2 core translation written in Rust built to work on the Casper Blockchain

## Done
- [x] Implemented the core contracts.
- [x] Implemented unit tests for the UniswapV2ERC20 contract.
- [x] Implemented unit tests for the required libraries.

## Todo
- [ ] Implement unit tests for the Factory and Pair contracts.
- [ ] Implement CASP-20 Dictionaries Version whenever that's released (doesn't impact interfaces).

## Install compilation target
Make sure `wasm32-unknown-unknown` is installed.
```bash
$ make prepare
```

## Build ERC20 and UniswapV2ERC20 contracts
```bash
$ make build-contract
```

## Test ERC20 and UniswapV2ERC20 contracts
Test logic and smart contracts.
```bash
$ make test
```

## Build all the Smart Contracts
```bash
$ cargo build
```

## Test Uniswap Libraries
```bash
$ cargo test -p uniswap-libs
```

## License
As this is a implementation of the Uniswap contracts, best guidance indicates mechanical translations inherit copyright elements of the reference code, which in this case is Uniswap V2. Portions of this code are unique and are to be released under the same license. As such, this is licensed as GPLv3
https://www.gnu.org/licenses/gpl-3.0.en.html

## Security
This code is and should be treated as insecure and exists as a reference implementation. This is due to the lack of testable references for potential vulnerabilities (no known equivalents of SWCs on Casper), and lack of production testing, code review, formal verification and audits. Like seriously, there's a lot you need to do to take this stuff to prod if you want to. 

## Credits
* This code is based upon Uniswap's AMM implementation Uniswap V2 Core
* Jihed Chalghaf from Bytecode Labs (an Arcadia Partner)
* Yassine Amor from Bytecode Labs and Arcadia
