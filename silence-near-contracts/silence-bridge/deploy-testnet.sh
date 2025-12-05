#!/bin/bash
# Deploy Silence Bridge to NEAR Testnet
# Requires: Docker daemon running, NEAR credentials configured

set -e

ACCOUNT_ID="${1:-silence-bridge.testnet}"
OWNER_ID="${2:-$ACCOUNT_ID}"
MIN_SOLVER_STAKE="${3:-1000000000000000000000000}"  # 1 NEAR in yoctoNEAR
PROTOCOL_FEE_BPS="${4:-50}"  # 0.5% in basis points

cd "$(dirname "$0")"

echo "=== Silence Bridge Deployment ==="
echo "Account ID: $ACCOUNT_ID"
echo "Owner ID: $OWNER_ID"
echo "Min Solver Stake: $MIN_SOLVER_STAKE yoctoNEAR (1 NEAR)"
echo "Protocol Fee: $PROTOCOL_FEE_BPS basis points (0.5%)"
echo ""

# Build using Docker
WASM_FILE="target/wasm32-unknown-unknown/release/silence_bridge_stripped.wasm"

if [ ! -f "$WASM_FILE" ]; then
    echo "Building contract with Docker..."
    ./docker-build.sh
fi

# Check if WASM exists
if [ ! -f "$WASM_FILE" ]; then
    echo "Error: WASM file not found at $WASM_FILE"
    echo "Run ./docker-build.sh first"
    exit 1
fi

# Check credentials
if [ ! -f "$HOME/.near-credentials/testnet/$ACCOUNT_ID.json" ]; then
    echo "Creating account $ACCOUNT_ID..."
    near account create-account sponsor-by-faucet-service $ACCOUNT_ID autogenerate-new-keypair save-to-legacy-keychain network-config testnet
fi

# Deploy
echo "Deploying contract..."
near contract deploy $ACCOUNT_ID \
  use-file "$WASM_FILE" \
  with-init-call new \
  json-args "{\"owner\": \"$OWNER_ID\", \"min_solver_stake\": \"$MIN_SOLVER_STAKE\", \"protocol_fee_bps\": $PROTOCOL_FEE_BPS}" \
  prepaid-gas '100.0 Tgas' \
  attached-deposit '0 NEAR' \
  network-config testnet \
  sign-with-keychain send

echo ""
echo "✅ Deployment complete!"
echo ""
echo "Verify deployment:"
echo "  near view $ACCOUNT_ID get_stats --network-id testnet"
echo ""
echo "Add to .env.local:"
echo "  NEXT_PUBLIC_SILENCE_BRIDGE_CONTRACT=$ACCOUNT_ID"

