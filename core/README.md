# ORE

**ORE is a fair-launch, proof-of-work, digital currency everyone can mine.**


## API
- [`Consts`](api/src/consts.rs) – Program constants.
- [`Entrypoint`](api/src/lib.rs) – The program entrypoint.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Event`](api/src/error.rs) – Custom program events.
- [`Instruction`](api/src/instruction.rs) – Declared instructions and arguments.

## Instructions
- [`Claim`](program/src/processor/claim.rs) – Distributes claimable rewards as tokens from the treasury to a miner.
- [`Close`](program/src/processor/close.rs) – Closes a proof account returns the rent to the owner.
- [`Crown`](program/src/processor/crown.rs) – Flags a proof account as the top staker on the network.
- [`Open`](program/src/processor/open.rs) – Creates a new proof account for a prospective miner.
- [`Mine`](program/src/processor/mine.rs) – Verifies a hash provided by a miner and issues claimable rewards.
- [`Stake`](program/src/processor/stake.rs) – Stakes ORE with a miner to increase their multiplier.
- [`Reset`](program/src/processor/reset.rs) – Resets the program for a new epoch.
- [`Update`](program/src/processor/update.rs) – Updates a proof account's miner authority.
- [`Upgrade`](program/src/processor/upgrade.rs) – Migrates ORE v1 tokens to ORE v2, one-for-one.
- [`Initialize`](program/src/processor/initialize.rs) – Initializes the Ore program, creating the bus, mint, and treasury accounts.

## State
 - [`Bus`](src/state/bus.rs) - An account (8 total) which tracks and limits the amount mined rewards each epoch.
 - [`Proof`](src/state/proof.rs) - An account (1 per miner) which tracks a miner's hash, claimable rewards, and lifetime stats.
 - [`Treasury`](src/state/treasury.rs) – A singleton account which manages program-wide variables and authorities.



## Tests

To run the test suite, use the Solana toolchain: 

```
cargo test-sbf
```

For line coverage, use llvm-cov:

```
cargo llvm-cov
```
