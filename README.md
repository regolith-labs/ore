# Ore

**Ore is a digital currency you can mine from anywhere, at home or on your phone.** It uses a novel proof-of-work algorithm to guarantee no miner can ever be starved out from earning rewards. 


## How it works

The primary innovation of Ore is to offer non-exclusive mining rewards. This means one miner winning rewards does not prevent another miner from winning also. Rather than setting up every miner in a winner-take-all competition against one another, Ore gives each miner their own individual computational challenge. As long as a miner provides a valid solution to their personal challenge, the protocol guarantees they will earn a piece of the supply. Since no miner can be censored from the network and valid solutions are non-exclusive, starvation is avoided.


## Supply

Ore protects holders from runaway supply inflation. Regardless of how many miners are active, the supply growth rate is strictly bounded to a range of `0 ≤ R ≤ 2 ORE/min`. In other words, linear. The reward rate – amount paid out to miners per valid hash – dynamically adjusts every 60 seconds to maintain a target average supply growth rate of `1 ORE/min`. This growth rate was chosen for its straightforward simplicity and predictability.


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
