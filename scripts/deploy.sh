#!/usr/bin/env bash
set -euo pipefail

# Example deployment helper for Stellar Tip Jar on testnet.
# Prerequisites:
# - `stellar` CLI installed and authenticated
# - testnet network configured in CLI
# - an account/funder alias configured (e.g., alice)

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACT_DIR="$ROOT_DIR/contracts/tipjar"

cd "$CONTRACT_DIR"

echo "Building contract WASM..."
stellar contract build

WASM_PATH="target/wasm32v1-none/release/tipjar.wasm"

echo "Deploying contract to Stellar testnet..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm "$WASM_PATH" \
  --source alice \
  --network testnet)

echo "Contract deployed with ID: $CONTRACT_ID"

echo "Initialize contract with token address (replace TOKEN_CONTRACT_ID):"
echo "stellar contract invoke --id $CONTRACT_ID --source alice --network testnet -- init --token TOKEN_CONTRACT_ID"

echo "Send a tip (replace addresses):"
echo "stellar contract invoke --id $CONTRACT_ID --source alice --network testnet -- tip --sender SENDER_ADDRESS --creator CREATOR_ADDRESS --amount 100"

echo "Check creator total tips:"
echo "stellar contract invoke --id $CONTRACT_ID --source alice --network testnet -- get_total_tips --creator CREATOR_ADDRESS"

echo "Withdraw creator balance:"
echo "stellar contract invoke --id $CONTRACT_ID --source CREATOR_ALIAS --network testnet -- withdraw --creator CREATOR_ADDRESS"
