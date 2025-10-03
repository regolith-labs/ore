# ORE

ORE is a crypto mining protocol.


## API
- [`Consts`](api/src/consts.rs) – Program constants.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Event`](api/src/error.rs) – Custom program events.
- [`Instruction`](api/src/instruction.rs) – Declared instructions and arguments.

## Instructions

#### Mining
- [`Automate`](program/src/automate.rs) - Configures a new automation.
- [`Checkpoint`](program/src/checkpoint.rs) - Checkpoints rewards from an prior round.
- [`ClaimORE`](program/src/claim_ore.rs) - Claims ORE mining rewards.
- [`ClaimSOL`](program/src/claim_sol.rs) - Claims SOL mining rewards.
- [`Deploy`](program/src/deploy.rs) – Deploys SOL to claim space on the board.
- [`Initialize`](program/src/initialize.rs) - Initializes program variables.
- [`Log`](program/src/log.rs) – Logs non-truncatable event data.
- [`Reset`](program/src/reset.rs) - Resets the board for a new round.
- [`Reset`](program/src/reset.rs) - Resets the board for a new round.

#### Staking
- [`Deposit`](program/src/deposit.rs) - Deposits ORE into a stake account.
- [`Withdraw`](program/src/withdraw.rs) - Withdraws ORE from a stake account.
- [`ClaimSeeker`](program/src/claim_seeker.rs) - Claims a Seeker genesis token. 
- [`ClaimYield`](program/src/claim_yield.rs) - Claims staking yield.

#### Admin
- [`Bury`](program/src/bury.rs) - Executes a buy-and-bury transaction.
- [`Wrap`](program/src/wrap.rs) - Wraps SOL in the treasury for swap transactions. 
- [`SetAdmin`](program/src/set_admin.rs) - Re-assigns the admin authority.
- [`SetFeeCollector`](program/src/set_admin.rs) - Updates the fee collection address.
- [`SetFeeRate`](program/src/set_admin.rs) - Updates the fee charged per swap.

## State
- [`Automation`](api/src/state/automation.rs) - Tracks automation configs. 
- [`Board`](api/src/state/board.rs) - Tracks the current round number and timestamps.
- [`Config`](api/src/state/config.rs) - Global program configs.
- [`Miner`](api/src/state/miner.rs) - Tracks a miner's game state.
- [`Round`](api/src/state/round.rs) - Tracks the game state of a given round.
- [`Seeker`](api/src/state/seeker.rs) - Tracks whether a Seeker token has been claimed.
- [`Stake`](api/src/state/stake.rs) - Manages a user's staking activity.
- [`Treasury`](api/src/state/treasury.rs) - Mints, burns, and escrows ORE tokens. 


## Tests

To run the test suite, use the Solana toolchain: 

```
cargo test-sbf
```

For line coverage, use llvm-cov:

```
cargo llvm-cov
```
