# ORE

**Digital gold, accelerated.**

## API
- [`Consts`](api/src/consts.rs) – Program constants.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Event`](api/src/error.rs) – Custom program events.
- [`Instruction`](api/src/instruction.rs) – Declared instructions and arguments.

## Instructions
- [`Bet`](program/src/bet.rs) - Creates a wager on the current block.
- [`Bury`](program/src/bury.rs) - Swaps bets into ORE and burns it.
- [`Close`](program/src/close.rs) - Closes a wager account.
- [`Initialize`](program/src/initialize.rs) - Initializes the program.
- [`Payout`](program/src/payout.rs) - Distributes the block reward to the winner.
- [`Reset`](program/src/reset.rs) - Resets the program for the next block.

## State
- [`Block`](api/src/state/block.rs) - A singleton account tracking rounds of wagering.
- [`Proof`](api/src/state/proof.rs) - (Deprecated) An account which tracks a miner's current hash and current stake.
- [`Treasury`](api/src/state/treasury.rs) – The ORE mint authority.
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
