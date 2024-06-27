# ORE

**ORE is a fair-launch, proof-of-work, digital currency anyone can mine.**


## Install

```sh
cargo install ore-cli
```


## Program
- [`Consts`](src/consts.rs) – Program constants.
- [`Entrypoint`](src/lib.rs) – The program entrypoint.
- [`Errors`](src/error.rs) – Custom program errors.
- [`Idl`](idl/ore.json) – Interface for clients, explorers, and programs.
- [`Instruction`](src/instruction.rs) – Declared instructions and arguments.
- [`Loaders`](src/loaders.rs) – Validation logic for loading Solana accounts.


## Instructions
- [`Reset`](src/processor/reset.rs) – Resets the program for a new epoch.
- [`Open`](src/processor/open.rs) – Creates a new proof account for a prospective miner.
- [`Close`](src/processor/close.rs) – Closes a new proof account returns the rent to the owner.
- [`Mine`](src/processor/mine.rs) – Verifies a hash provided by a miner and issues claimable rewards.
- [`Stake`](src/processor/stake.rs) – Stakes ORE with a miner to increase their multiplier.
- [`Claim`](src/processor/claim.rs) – Distributes claimable rewards as tokens from the treasury to a miner.
- [`Upgrade`](src/processor/upgrade.rs) – Migrates v1 ORE tokens to v2 ORE.
- [`Initialize`](src/processor/initialize.rs) – Initializes the Ore program, creating the bus, mint, and treasury accounts.


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
