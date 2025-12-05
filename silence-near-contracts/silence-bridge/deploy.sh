#!/bin/bash
# Deploy Silence Bridge contract to NEAR Testnet

set -e

WASM_FILE="target/wasm32-unknown-unknown/release/silence_bridge.wasm"
ACCOUNT_ID="${1:-silence-bridge.testnet}"
OWNER_ID="${2:-$ACCOUNT_ID}"
MIN_SOLVER_STAKE="${3:-1000000000000000000000000}"  # 1 NEAR in yoctoNEAR
PROTOCOL_FEE_BPS="${4:-50}"  # 0.5% in basis points

echo "Deploying Silence Bridge contract..."
echo "Account ID: $ACCOUNT_ID"
echo "Owner ID: $OWNER_ID"
echo "Min Solver Stake: $MIN_SOLVER_STAKE yoctoNEAR (1 NEAR)"
echo "Protocol Fee: $PROTOCOL_FEE_BPS basis points (0.5%)"
echo ""

# Check if WASM file exists
if [ ! -f "$WASM_FILE" ]; then
    echo "Error: WASM file not found at $WASM_FILE"
    echo "Building contract first..."
    cargo build --target wasm32-unknown-unknown --release
fi

# Check if account exists, if not provide instructions
if [ ! -f "$HOME/.near-credentials/testnet/$ACCOUNT_ID.json" ]; then
    echo "⚠️  Account $ACCOUNT_ID not found!"
    echo ""
    echo "To create the account, run:"
    echo "  near account create-account sponsor-by-faucet-service $ACCOUNT_ID autogenerate-new-keypair save-to-keychain"
    echo ""
    echo "Or if you have a parent account (e.g., your-account.testnet), create a subaccount:"
    echo "  near account create-account fund-myself $ACCOUNT_ID autogenerate-new-keypair save-to-keychain"
    echo ""
    echo "Then run this script again."
    exit 1
fi

# Deploy contract
near deploy \
  --wasm-file "$WASM_FILE" \
  --init-function new \
  --init-args "{\"owner_id\": \"$OWNER_ID\", \"min_solver_stake\": \"$MIN_SOLVER_STAKE\", \"protocol_fee_bps\": $PROTOCOL_FEE_BPS}" \
  --network-id testnet \
  "$ACCOUNT_ID"

echo ""
echo "✅ Deployment complete!"
echo "Contract deployed to: $ACCOUNT_ID"
echo ""
echo "Add to your .env.local:"
echo "NEXT_PUBLIC_SILENCE_BRIDGE_CONTRACT=$ACCOUNT_ID"
