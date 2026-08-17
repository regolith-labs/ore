# DiscretionaryBps Automation Strategy

## Overview

`DiscretionaryBps` (strategy ID `3`) is a new automation strategy for third-party mining executors. It is functionally identical to `Discretionary` (strategy ID `2`) with one difference: the executor fee is expressed as **basis points (BPS)** of the total deployed amount, rather than a fixed lamport value.

## Motivation

With the `Discretionary` strategy, the executor fee is a fixed number of lamports per round. This creates friction when deployment amounts vary — a fixed fee that is fair for a 1 SOL deployment may be too large for a 0.01 SOL deployment, or too small for a 10 SOL deployment.

`DiscretionaryBps` solves this by making the fee proportional to the deployment size. An executor charging 50 BPS (0.5%) takes 0.005 SOL on a 1 SOL deployment and 0.00005 SOL on a 0.01 SOL deployment.

## How it works

### Fee calculation

The fee is calculated as:

```
executor_fee = (total_deployed_amount * fee_bps) / 10,000
```

Where:
- `total_deployed_amount` = `amount` per square x number of squares deployed to
- `fee_bps` = the `fee` field on the Automation account (interpreted as basis points)
- `10,000` = the BPS denominator (`DENOMINATOR_BPS`)

The calculation uses u128 intermediate arithmetic to prevent overflow.

### Constraints

- **Maximum fee: 100 BPS (1%).** The program enforces `fee <= 100` when creating or updating a `DiscretionaryBps` automation. Attempting to set a higher value will fail.
- **Permissionless executor blocked.** Like `Discretionary`, `DiscretionaryBps` cannot use the protocol's permissionless executor (`EXECUTOR_ADDRESS`). The automation must specify a custom executor.

### When the fee is charged

The fee is charged **once per round**, on the miner's first deploy of that round. Subsequent deploys in the same round do not incur an additional fee. This matches the behavior of `Discretionary`.

### Executor control

Like `Discretionary`, the executor has full control over:
- **Amount**: the executor provides the `amount` in the Deploy instruction data, capped at `automation.amount`
- **Tile selection**: the executor provides the `squares` bitmask in the Deploy instruction data

## Integration

### Creating a DiscretionaryBps automation

Use the `automate` SDK function with `strategy = 3`:

```rust
use ore_api::sdk::automate;
use ore_api::state::AutomationConditions;

let ix = automate(
    signer,              // authority (miner owner)
    amount,              // max SOL per square per round
    deposit,             // SOL to deposit into automation balance
    executor,            // your executor pubkey
    50,                  // fee: 50 BPS = 0.5%
    0,                   // mask (unused for discretionary — executor provides at deploy time)
    3,                   // strategy: DiscretionaryBps
    true,                // reload: auto-reload SOL rewards into automation balance
    AutomationConditions::default(),
);
```

### Deploying on behalf of a miner

The Deploy instruction is identical to `Discretionary`. The executor submits a Deploy transaction with:
- `amount`: lamports per square (capped at `automation.amount`)
- `squares`: a 32-bit bitmask where bits 0-24 select which squares to deploy to

```rust
use ore_api::sdk::deploy;

let ix = deploy(
    executor_pubkey,    // signer (executor)
    miner_authority,    // the miner's authority
    amount,             // lamports per square
    squares_bitmask,    // which squares to deploy to
);
```

### Balance management

The automation account tracks its SOL balance in the `balance` field. On each deploy:

1. **Sufficiency check**: if `balance < (amount * num_squares) + fee`, the automation is closed and remaining balance is returned to the authority.
2. **Fee deduction**: `balance -= total_deployed + executor_fee`
3. **Low balance close**: if `balance < amount + min_fee(amount)`, the automation is closed after the deploy.

For `DiscretionaryBps`, the fee estimates in steps 1 and 3 use the same BPS formula as the actual fee in step 2, ensuring consistent behavior.

### Differences from Discretionary

| | Discretionary | DiscretionaryBps |
|---|---|---|
| Strategy ID | 2 | 3 |
| Fee field interpretation | Fixed lamports | Basis points (1 BPS = 0.01%) |
| Fee range | Any u64 | 0 - 100 (0% - 1%) |
| Fee calculation | `fee` | `(total_deployed * fee) / 10,000` |
| Executor control | Full (amount + tiles) | Full (amount + tiles) |
| Permissionless executor | Blocked | Blocked |

### Example fee amounts

| fee (BPS) | Deployment | Executor receives |
|-----------|-----------|------------------|
| 10 | 0.01 SOL | 0.00001 SOL |
| 10 | 1 SOL | 0.001 SOL |
| 50 | 0.01 SOL | 0.00005 SOL |
| 50 | 1 SOL | 0.005 SOL |
| 100 | 0.01 SOL | 0.0001 SOL |
| 100 | 1 SOL | 0.01 SOL |
