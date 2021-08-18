# uniswap-v2-core
Uniswap V2 core written in Rust.

## Done
- [x] Implemented the core contracts.
- [x] Implemented unit tests for the UniswapV2ERC20 contract.
- [x] Implemented unit tests for the required libraries.

## Todo
- [ ] Implement unit tests for the Factory and Pair contracts.

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