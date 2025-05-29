# ORE

**Digital gold, accelerated.**

## API
- [`Consts`](api/src/consts.rs) – Program constants.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Event`](api/src/error.rs) – Custom program events.
- [`Instruction`](api/src/instruction.rs) – Declared instructions and arguments.

## Instructions
- [`Bury`](program/src/bury.rs) - Swap committed tokens into ORE and burns it.
- [`Close`](program/src/close.rs) - Close a commit account.
- [`Deploy`](program/src/deploy.rs) - Deploy capital to mine the current block.
- [`Initialize`](program/src/initialize.rs) - Initialize the program.
- [`Payout`](program/src/payout.rs) - Payout the block reward to the winning commit.
- [`Reset`](program/src/reset.rs) - Start the next block.

## State
- [`Block`](api/src/state/block.rs) - A singleton account tracking rounds of commits.
- [`Proof`](api/src/state/proof.rs) - (Deprecated) An account which tracks a miner's current hash and current stake.
- [`Treasury`](api/src/state/treasury.rs) – The mint authority on the ORE token.
- [`Commit`](api/src/state/commit.rs) - Capital deployed by a miner in the current block.


## Tests

To run the test suite, use the Solana toolchain: 

```
cargo test-sbf
```

For line coverage, use llvm-cov:

```
cargo llvm-cov
```
