# ORE

**Mine. Trade. Win.**

## Summary

ORE is a cryptocurrency mining and trading game. Miners compete to earn block rewards by buying and selling hashpower in an open market. The more hashpower you acquire, the greater your chances of generating the best hash and winning the block reward. Will you hit the motherlode?


## API
- [`Consts`](api/src/consts.rs) – Program constants.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Event`](api/src/error.rs) – Custom program events.
- [`Instruction`](api/src/instruction.rs) – Declared instructions and arguments.

## Instructions

#### User
- [`Claim`](program/src/claim.rs) - Claims miner rewards. 
- [`Close`](program/src/close.rs) - Closes a block account.
- [`Initialize`](program/src/initialize.rs) - Initializes the program.
- [`Log`](program/src/log.rs) – Logs events as non-truncatable data.
- [`Mine`](program/src/mine.rs) - Submits hashes for scoring.
- [`Open`](program/src/open.rs) - Opens a new block.
- [`Reset`](program/src/reset.rs) – Resets the hashpower market for the next block.
- [`Swap`](program/src/swap.rs) - Executes a buy or sell in the hashpower market.

#### Admin
- [`SetAdmin`](program/src/set_admin.rs) - Re-assigns the admin authority.
- [`SetFeeCollector`](program/src/set_admin.rs) - Updates the fee collection address.
- [`SetFeeRate`](program/src/set_admin.rs) - Updates the fee charged per swap.

## State
- [`Block`](api/src/state/block.rs) - A round in the game.
- [`Config`](api/src/state/config.rs) - Global program configuration.
- [`Market`](api/src/state/market.rs) - Hashpower market.
- [`Miner`](api/src/state/miner.rs) - Tracks a miner state and history.
- [`Treasury`](api/src/state/treasury.rs) - The mint authority of the ORE token.


## Block lifecycle

- `Open` a block with an upcoming ID.
- `Reset` to move the market forward, and begin trading. 
- `Swap` to trade hashpower in the market for the current block.
- Trading ends automatically once the end slot is reached.
- `Reset` to move market forward again, and record the slot hash for the ended block.
- `Mine` to submit the best hash within your available nonce range.
- `Close` to return rent and transfer block reward to the winning miner for claiming.
- `Claim` to claim block reward.


## Tests

To run the test suite, use the Solana toolchain: 

```
cargo test-sbf
```

For line coverage, use llvm-cov:

```
cargo llvm-cov
```
