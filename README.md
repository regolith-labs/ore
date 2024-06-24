# ORE

**ORE is a fair-launch, proof-of-work, cross-border digital currency anyone can mine.**

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
- [`Initialize`](src/processor/initialize.rs) – Initializes the Ore program, creating the bus, mint, and treasury accounts.
- [`Reset`](src/processor/reset.rs) – Resets the program for a new epoch.
- [`Register`](src/processor/register.rs) – Creates a new proof account for a prospective miner.
- [`Mine`](src/processor/mine.rs) – Verifies a hash provided by a miner and issues claimable rewards.
- [`Claim`](src/processor/claim.rs) – Distributes claimable rewards as tokens from the treasury to a miner.
- [`UpdateAdmin`](src/processor/update_admin.rs) – Updates the admin authority.
- [`UpdateDifficulty`](src/processor/update_difficulty.rs) - Updates the hashing difficulty.


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
