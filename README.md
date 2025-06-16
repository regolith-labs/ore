# ORE

**Mine blockspace. Trade hashpower. Earn rewards.**

## API
- [`Consts`](api/src/consts.rs) – Program constants.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Event`](api/src/error.rs) – Custom program events.
- [`Instruction`](api/src/instruction.rs) – Declared instructions and arguments.

## Instructions

- [`Open`](program/src/open.rs) - Open a new block.
- [`Close`](program/src/close.rs) - Close a block and pay out the reward.
- [`Mine`](program/src/mine.rs) - Mine the current block.
- [`Swap`](program/src/swap.rs) - Trade in a hashpower market.

## State
- [`Block`](api/src/state/block.rs) - A period of time for mining.
- [`Config`](api/src/state/config.rs) - Global program configuration.
- [`Market`](api/src/state/market.rs) - Hashpower market for a given block.
- [`Miner`](api/src/state/miner.rs) - Tracks a miner state and history.
- [`Permit`](api/src/state/permit.rs) - Tracks a miner's commitment to mine a block.
- [`Stake`](api/src/state/stake.rs) - Tracks a miner's collateral for trading in a market.
- [`Treasury`](api/src/state/treasury.rs) - The mint authority of the ORE token.


## Tests

To run the test suite, use the Solana toolchain: 

```
cargo test-sbf
```

For line coverage, use llvm-cov:

```
cargo llvm-cov
```

