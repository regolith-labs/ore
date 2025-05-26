# ORE

**Digital gold, accelerated.**

## API
- [`Consts`](api/src/consts.rs) – Program constants.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Event`](api/src/error.rs) – Custom program events.
- [`Instruction`](api/src/instruction.rs) – Declared instructions and arguments.

## Instructions
- [`Bet`](program/src/bet.rs) - Open a wager.
- [`Bury`](program/src/bury.rs) - Swap wagered tokens into ORE and burns it.
- [`Close`](program/src/close.rs) - Close a wager account.
- [`Initialize`](program/src/initialize.rs) - Initialize the program.
- [`Payout`](program/src/payout.rs) - Payout the block reward to the winning wager.
- [`Reset`](program/src/reset.rs) - Start the next block.

## State
- [`Block`](api/src/state/block.rs) - A singleton account tracking rounds of wagering.
- [`Proof`](api/src/state/proof.rs) - (Deprecated) An account which tracks a miner's current hash and current stake.
- [`Treasury`](api/src/state/treasury.rs) – The mint authority on the ORE token.
- [`Wager`](api/src/state/wager.rs) - A bet placed by a user.


## Tests

To run the test suite, use the Solana toolchain: 

```
cargo test-sbf
```

For line coverage, use llvm-cov:

```
cargo llvm-cov
```
