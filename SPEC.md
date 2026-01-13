# Top Miner Verification in Reset Instruction

## Overview

Add verification logic to the `reset` instruction that validates the top miner account passed in by the backend matches the expected winner. This is a **dry-run phase** for testing backend infrastructure - verification failures should be logged but must not cause transaction failures.

## Background

The backend now pre-computes the top miner off-chain and passes it into the `reset` instruction. The backend uses identical RNG logic (slot_hash XOR, bit reversal, modulo deployed) and iterates through all miners' cumulative ranges to find the winner.

## Implementation Details

### File Location
- **File**: `program/src/reset.rs`
- **Insertion Point**: Line 236 (existing placeholder with commented-out code)

### Account Changes
- Rename `_top_miner_info` to `top_miner_info` (line 15) since it's now being used

### Verification Logic

The verification should:

1. **Guard against split rewards**: Only run if `round.top_miner != SPLIT_ADDRESS`
   - If split reward, skip verification entirely

2. **Parse the miner account**: Use steel's `as_account::<Miner>(&ore_api::ID)`
   - On deserialization failure: silently continue (no log, no error)

3. **Validate round ID**: Assert `miner.round_id == round.id`
   - On mismatch: silently continue

4. **Calculate sample point**: `round.top_miner_sample(r, winning_square)`

5. **Verify cumulative range**:
   ```rust
   top_miner_sample >= miner.cumulative[winning_square]
       && top_miner_sample < miner.cumulative[winning_square] + miner.deployed[winning_square]
   ```

6. **Log results**:
   - On success: `sol_log("Top miner verified")`
   - On failure: `sol_log("verification failed")`

### Cases Where Verification is Skipped

| Condition | Reason | Handling |
|-----------|--------|----------|
| No RNG (`round.rng()` returns None) | No randomness available | Already returns early at line 92-125 |
| No winners (`deployed[winning_square] == 0`) | No one deployed to winning square | Already returns early at line 134-170 |
| Split reward (`round.top_miner == SPLIT_ADDRESS`) | Rewards split among all miners | Explicit guard check before verification |
| Deserialization failure | Invalid/wrong account passed | Silent continue |
| Round ID mismatch | Miner not from current round | Silent continue |

### Error Handling

**Critical requirement**: No panics or transaction failures from this verification.

- All failures must be handled gracefully
- Use `if let Ok(...)` patterns instead of `?` operator for fallible operations
- Log failures minimally using `sol_log`

### Data Sources

| Data | Source | Notes |
|------|--------|-------|
| RNG value `r` | `round.rng()` | Available after line 92 |
| `winning_square` | `round.winning_square(r)` | Calculated at line 131 |
| `top_miner_sample` | `round.top_miner_sample(r, winning_square)` | Method on Round |
| Miner cumulative/deployed | Parsed from `top_miner_info` | Arrays of u64[25] |

### Pseudocode

```rust
// Validate top miner (dry-run - no errors on failure)
if round.top_miner != SPLIT_ADDRESS {
    if let Ok(miner) = top_miner_info.as_account::<Miner>(&ore_api::ID) {
        if miner.round_id == round.id {
            let top_miner_sample = round.top_miner_sample(r, winning_square);
            if top_miner_sample >= miner.cumulative[winning_square]
                && top_miner_sample < miner.cumulative[winning_square] + miner.deployed[winning_square]
            {
                sol_log("Top miner verified");
            } else {
                sol_log("verification failed");
            }
        }
    }
}
```

## Future Considerations

- **Enforcement phase**: Will be enabled via new program deployment (not a config flag)
- **No storage needed**: Don't store verified address; just log for now
- **No Round updates**: Don't set `round.top_miner` yet - that happens during checkpoint

## Testing

The backend team will monitor logs to verify their infrastructure correctly computes the top miner. Success criteria:
- "Top miner verified" logs appear consistently when backend passes correct miner
- No increase in transaction failures after deployment
