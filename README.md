# ORE

**Mine blockspace. Trade hashpower. Win rewards.**

## API
- [`Consts`](api/src/consts.rs) – Program constants.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Event`](api/src/error.rs) – Custom program events.
- [`Instruction`](api/src/instruction.rs) – Declared instructions and arguments.

## Instructions

#### Mine
- [`Open`](program/src/open.rs) - Opens a new block for mining.
- [`Close`](program/src/close.rs) - Closes a block and pays out rewards.
- [`Mine`](program/src/mine.rs) - Mines the current block by computing hashes.

#### Stake
- [`Deposit`](program/src/deposit.rs) - Deposits stake into a miner account.
- [`Withdraw`](program/src/withdraw.rs) - Withdraws stake from a miner account.
- [`Free`](program/src/free.rs) - Frees up miner capacity after block ends.

#### Trade
- [`Buy`](program/src/buy.rs) - Buys hash tokens from the market.
- [`Sell`](program/src/sell.rs) - Sells hash tokens to the market.

## State
- [`Block`](api/src/state/block.rs) - A period of time for mining.
- [`Config`](api/src/state/config.rs) - Global program configuration.
- [`Market`](api/src/state/market.rs) - Hashpower market for a given block.
- [`Miner`](api/src/state/miner.rs) - A user's mining and staking state.
- [`Receipt`](api/src/state/receipt.rs) - Tracks a miner's deployed capital.
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
