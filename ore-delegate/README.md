# ORE Delegate

**Delegate mining to another party.**

## API
- [`Consts`](api/src/consts.rs) – Program constants.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Event`](api/src/error.rs) – Custom program events.
- [`Instruction`](api/src/instruction.rs) – Declared instructions and arguments.

## Instructions
- [`Deposit`](program/src/deposit.rs) - Deposit hash tokens.
- [`Withdraw`](program/src/withdraw.rs) - Withdraw hash tokens.
- [`Crank`](program/src/crank.rs) - Crank mine instructions.
- [`Payout`](program/src/payout.rs) - Payout mining rewards.

## State
- [`Delegate`](api/src/state/delegate.rs) - Escrows hash tokens on behalf of a miner.


## Tests

To run the test suite, use the Solana toolchain: 

```
cargo test-sbf
```

For line coverage, use llvm-cov:

```
cargo llvm-cov
```
