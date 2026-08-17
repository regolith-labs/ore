# Solana Program Deployment Runbook

This is an interactive deployment runbook designed to be executed by Claude. Each step should be run sequentially, with Claude providing output and guidance.

## Prerequisites

- `solana-verify` CLI installed
- `solana` CLI installed and configured
- `vbi` (Verify Buffer Integrity) tool installed
- Environment variable `HELIUS_API_KEY` set with your Helius API key

## Configuration

**Update these variables for your program before running the deployment:**

```bash
# Program configuration - EDIT THESE FOR YOUR PROGRAM
PROGRAM_ID="oreV3EG1i9BEgiAJ8b177Z2S2rMarzak4NMv1kULvWv"
MULTISIG_AUTHORITY="7eyKjpFyTszL1JB1g3BAi7UucCQfKHPThX56ehZ9kxAh"
GITHUB_REPO="https://github.com/regolith-labs/ore"
LIBRARY_NAME="ore"
```

| Variable | Description |
|----------|-------------|
| `PROGRAM_ID` | The on-chain program address |
| `MULTISIG_AUTHORITY` | The Squads multisig address that controls program upgrades |
| `GITHUB_REPO` | The GitHub repository URL for the program source |
| `LIBRARY_NAME` | The library name (used for binary name and temp files) |

---

## Deployment Steps

### Step 0: Review changes to be deployed

Fetch the currently deployed commit from Solana Verify and summarize all changes.

**Fetch deployed commit:**

```bash
DEPLOYED_COMMIT=$(curl -s "https://verify.osec.io/status/${PROGRAM_ID}" \
  -H "Accept: application/json" | grep -o '"commit":"[^"]*"' | cut -d'"' -f4)
echo "Currently deployed commit: $DEPLOYED_COMMIT"
```

**Show commits since deployed version:**

```bash
git log --oneline $DEPLOYED_COMMIT..HEAD
```

**Show files changed:**

```bash
git diff --stat $DEPLOYED_COMMIT..HEAD
```

**Show code changes:**

```bash
git diff $DEPLOYED_COMMIT..HEAD
```

**Claude**: Present a clear summary of:
1. Number of commits being deployed
2. List of commits with their messages
3. Files changed and their purpose
4. Any breaking changes or risk assessment

Then use `AskUserQuestion` to confirm the user wants to proceed with deployment.

---

### Step 1: Build the program

Build the program with solana-verify to ensure reproducible builds.

```bash
solana-verify build
```

**Expected output**: Build completes successfully, `target/deploy/${LIBRARY_NAME}.so` is created.

---

### Step 2: Generate temporary buffer keypair

Create a new keypair for the program buffer. This keypair's address will be the buffer address.

```bash
BUFFER_KEYPAIR="/tmp/${LIBRARY_NAME}-buffer-$(date +%s).json"
solana-keygen new -o "$BUFFER_KEYPAIR" --no-bip39-passphrase --force
```

**Save the keypair path and get the buffer address**:

```bash
BUFFER_ADDRESS=$(solana address -k "$BUFFER_KEYPAIR")
echo "Buffer keypair: $BUFFER_KEYPAIR"
echo "Buffer address: $BUFFER_ADDRESS"
```

---

### Step 3: Write program to buffer

Deploy the program binary to the buffer account on mainnet.

```bash
solana program write-buffer "target/deploy/${LIBRARY_NAME}.so" \
  --buffer "$BUFFER_KEYPAIR" \
  --with-compute-unit-price 1000000 \
  --url "https://mainnet.helius-rpc.com/?api-key=${HELIUS_API_KEY}"
```

**Expected output**: Buffer write completes, confirms the buffer address.

---

### Step 4: Verify buffer integrity

Run VBI to verify the buffer matches the local build.

```bash
vbi --program-file "target/deploy/${LIBRARY_NAME}.so" --buffer-address "$BUFFER_ADDRESS"
```

**Expected output**: Verification passes, confirming buffer integrity.

---

### Step 5: Export PDA verification transaction

Generate the verification transaction for Solana Verify.

```bash
solana-verify export-pda-tx \
  "$GITHUB_REPO" \
  --library-name "$LIBRARY_NAME" \
  --program-id "$PROGRAM_ID" \
  --uploader "$MULTISIG_AUTHORITY" \
  --encoding base58 \
  --compute-unit-price 0
```

**Expected output**: Base58-encoded transaction data printed to console.

---

### Step 6: Output summary

Print final summary with all relevant addresses and commit URL.

```bash
COMMIT_HASH=$(git rev-parse HEAD)
REMOTE_URL=$(git remote get-url origin | sed 's/\.git$//' | sed 's|git@github.com:|https://github.com/|')
echo ""
echo "========== DEPLOYMENT SUMMARY =========="
echo "Buffer Address: $BUFFER_ADDRESS"
echo "Solana Address: $(solana address)"
echo "Commit: ${REMOTE_URL}/commit/${COMMIT_HASH}"
echo "========================================"
```

---

### Step 7: Set buffer authority to multisig

**Manual step required**: Go to the Squads multisig app and create the deployment transaction.

1. Open Squads and navigate to your multisig
2. Create a new program upgrade transaction using the buffer address from the summary above
3. Once the transaction is created in Squads, return here and confirm

**Claude**: Use `AskUserQuestion` to prompt the user to confirm when they have created the deployment in Squads.

After user confirms, run the following command to transfer buffer authority to the multisig:

```bash
solana program set-buffer-authority "$BUFFER_ADDRESS" \
  --new-buffer-authority "$MULTISIG_AUTHORITY" \
  -um
```

**Expected output**: Buffer authority updated successfully.

---

### Step 8: Wait for multisig approval

**Manual step required**: All multisig signers must approve the deployment in Squads.

1. Notify all multisig signers that the deployment is ready for approval
2. Each signer must go to Squads and sign the deployment transaction
3. Once all required signatures are collected, the deployment will execute automatically

**Claude**: Use `AskUserQuestion` to prompt the user to confirm when the multisig deployment has been fully executed.

---

### Step 9: Cleanup temporary keypair

Once the deployment is fully complete, clean up the temporary keypair.

```bash
rm -f "$BUFFER_KEYPAIR"
echo "Cleaned up temporary buffer keypair"
```

---

### Step 10: Submit verification job

Submit a verification job to confirm the on-chain program matches the GitHub source.

**Note**: This command may fail due to rate limits or temporary hash mismatches. Retry until successful.

```bash
solana-verify remote submit-job \
  --program-id "$PROGRAM_ID" \
  --uploader "$MULTISIG_AUTHORITY"
```

**Claude**: If this command fails, wait a few seconds and retry. Continue retrying until the job is successfully submitted.

**Expected output**: Verification job submitted successfully.

---

## Notes for Claude

When executing this runbook:

**General:**
- Run each step sequentially using the Bash tool
- If any step fails unexpectedly, stop and report the error before proceeding
- Verify `HELIUS_API_KEY` environment variable is set before Step 3
- Read the Configuration section variables and use them in all commands

**Step-specific instructions:**
- **Step 0**: Fetch the deployed commit from the Solana Verify API using `$PROGRAM_ID`, present a clear summary of all changes, and use `AskUserQuestion` to confirm before proceeding
- **Step 2**: Store `BUFFER_KEYPAIR` and `BUFFER_ADDRESS` for use in subsequent steps
- **Step 5**: Present the base58 transaction output in a clearly copyable format
- **Step 6**: The commit URL is derived from git remote
- **Step 7**: Use `AskUserQuestion` to prompt the user to confirm they've created the Squads deployment before running the set-buffer-authority command
- **Step 8**: Use `AskUserQuestion` to prompt the user to confirm the multisig deployment has been fully executed
- **Step 9**: Only run cleanup after user confirms deployment is complete
- **Step 10**: Retry the verification job command until it succeeds (may fail due to rate limits or temporary issues)
