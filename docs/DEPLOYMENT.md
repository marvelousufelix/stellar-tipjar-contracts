# Deployment Guide

End-to-end guide for building, testing, and deploying the TipJar contract to testnet and mainnet.

---

## Prerequisites

| Requirement | Notes |
|---|---|
| Rust toolchain (stable) | `rustup update stable` |
| `wasm32v1-none` target | `rustup target add wasm32v1-none` |
| Stellar CLI | [Install guide](https://developers.stellar.org/docs/tools/developer-tools/cli/install-cli) |
| `jq` | Used by deploy scripts for config updates |
| Funded Stellar account | Testnet: use Friendbot; Mainnet: fund via exchange |

---

## Build

```bash
# Build optimized WASM
cargo build -p tipjar --target wasm32v1-none --release

# Optimize WASM size (reduces deployment cost)
stellar contract optimize --wasm target/wasm32v1-none/release/tipjar.wasm
# Output: target/wasm32v1-none/release/tipjar.optimized.wasm
```

---

## Test Before Deploying

```bash
# Unit tests
cargo test -p tipjar

# Full integration test suite
cargo test --workspace
```

All tests must pass before proceeding to deployment.

---

## Testnet Deployment

### 1. Set environment variables

```bash
export DEPLOYER_SECRET=<your-testnet-secret-key>
```

### 2. Run the deploy script

```bash
bash scripts/deploy_testnet.sh
```

This script:
1. Builds and optimizes the WASM
2. Deploys to testnet via `stellar contract deploy`
3. Verifies the contract is live
4. Runs smoke tests
5. Records the contract ID in `deployment/config.json`

### 3. Initialize the contract

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source $DEPLOYER_SECRET \
  --network testnet \
  -- init \
  --admin ADMIN_ADDRESS \
  --token TOKEN_ADDRESS
```

### 4. Whitelist additional tokens (if needed)

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source $DEPLOYER_SECRET \
  --network testnet \
  -- add_token \
  --admin ADMIN_ADDRESS \
  --token ADDITIONAL_TOKEN_ADDRESS
```

### 5. Verify deployment

```bash
bash scripts/verify_deployment.sh $CONTRACT_ID testnet
```

---

## Mainnet Deployment

> Complete the [Mainnet Readiness Checklist](MAINNET_CHECKLIST.md) before proceeding.

### 1. Set environment variables

```bash
export DEPLOYER_SECRET=<your-mainnet-secret-key>
```

### 2. Run the mainnet deploy script

```bash
bash scripts/deploy-mainnet.sh
# or equivalently:
bash scripts/deploy_mainnet.sh
```

The script requires interactive confirmation (`deploy mainnet`) unless `CI_MAINNET_CONFIRMED=true` is set.

### 3. Initialize the contract

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source $DEPLOYER_SECRET \
  --network mainnet \
  -- init \
  --admin ADMIN_ADDRESS \
  --token TOKEN_ADDRESS
```

### 4. Post-deployment validation

```bash
# Verify contract is live
bash scripts/verify_deployment.sh $CONTRACT_ID mainnet

# Check contract version
stellar contract invoke \
  --id $CONTRACT_ID \
  --network mainnet \
  -- get_contract_version

# Test a read-only query
stellar contract invoke \
  --id $CONTRACT_ID \
  --network mainnet \
  -- get_total_tips \
  --creator ADMIN_ADDRESS \
  --token TOKEN_ADDRESS
```

---

## Configuration

Contract IDs and deployment history are stored in `deployment/config.json`:

```json
{
  "networks": {
    "testnet": { "active_contract_id": "C..." },
    "mainnet": { "active_contract_id": "C..." }
  },
  "history": [...]
}
```

---

## Rollback Procedure

If a critical issue is found after deployment:

1. **Pause the contract immediately** to stop fund movement:
   ```bash
   stellar contract invoke \
     --id $CONTRACT_ID \
     --source $DEPLOYER_SECRET \
     --network mainnet \
     -- pause \
     --caller ADMIN_ADDRESS \
     --reason "Critical issue detected - investigating"
   ```

2. **Investigate** the issue. Check events via the indexer or Stellar Explorer.

3. **Roll back** the active contract ID to the previous deployment:
   ```bash
   bash scripts/rollback.sh mainnet
   ```
   This updates `deployment/config.json` to point to the previous contract ID. Direct users to the previous contract while the fix is prepared.

4. **Deploy a fix** using the standard deployment flow once the issue is resolved.

5. **Unpause** (if the new contract was paused, not the rolled-back one):
   ```bash
   stellar contract invoke \
     --id $NEW_CONTRACT_ID \
     --source $DEPLOYER_SECRET \
     --network mainnet \
     -- unpause \
     --caller ADMIN_ADDRESS
   ```

---

## Monitoring Setup

### Prometheus + Grafana

```bash
# Start monitoring stack
docker-compose -f monitoring/docker-compose.yml up -d
```

- Prometheus config: `monitoring/prometheus/prometheus.yml`
- Alert rules: `monitoring/prometheus/alert_rules.yml`
- Grafana dashboard: `monitoring/grafana/contract_health.json`

### Event Monitor

```bash
bash scripts/start_indexer.sh
```

Key alerts configured:
- High transaction volume (>100 tx/5min)
- Error rate >5%
- Excessive gas usage
- Indexer lag >100 ledgers

---

## Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common errors and fixes.

### Build fails: `can't find crate for 'std'`

```bash
rustup target add wasm32v1-none
```

### Deploy fails: `DEPLOYER_SECRET not set`

```bash
export DEPLOYER_SECRET=<your-secret-key>
```

### Contract not found after deploy

Confirm the `--network` flag matches where you deployed. Check `deployment/config.json` for the recorded contract ID.

### Stale test snapshots

```bash
rm -rf contracts/tipjar/test_snapshots
cargo test -p tipjar
```
