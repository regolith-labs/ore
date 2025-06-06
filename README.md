# ORE

**Mine blockspace. Trade hashpower. Win rewards.**

## API
- [`Consts`](api/src/consts.rs) – Program constants.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Event`](api/src/error.rs) – Custom program events.
- [`Instruction`](api/src/instruction.rs) – Declared instructions and arguments.

## Instructions

- [`Open`](program/src/open.rs) - Opens a new block for mining.
- [`Close`](program/src/close.rs) - Closes a block and pays out rewards.
- [`Mine`](program/src/mine.rs) - Mines the current block by computing hashes.
- [`Swap`](program/src/swap.rs) - Trade in a hashpower market.

## State
- [`Block`](api/src/state/block.rs) - A period of time for mining.
- [`Config`](api/src/state/config.rs) - Global program configuration.
- [`Market`](api/src/state/market.rs) - Hashpower market for a given block.
- [`Miner`](api/src/state/miner.rs) - Tracks a user's mining state.
- [`Treasury`](api/src/state/treasury.rs) - The mint authority on the ORE token.


## Tests

To run the test suite, use the Solana toolchain: 

```
cargo test-sbf
```

For line coverage, use llvm-cov:

```
cargo llvm-cov
```
