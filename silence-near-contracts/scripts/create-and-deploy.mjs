import { KeyPair, keyStores, connect, Account, Contract } from "near-api-js";
import { join } from "path";
import { homedir } from "os";
import { existsSync, mkdirSync, writeFileSync, readFileSync } from "fs";

const ACCOUNT_ID = "silence-bridge-v2.testnet";
const CREDENTIALS_DIR = join(homedir(), ".near-credentials", "testnet");
const WASM_PATH = "/tmp/silence_bridge_clean.wasm";

async function main() {
  console.log("Creating NEAR testnet account:", ACCOUNT_ID);
  
  // Ensure credentials directory exists
  if (!existsSync(CREDENTIALS_DIR)) {
    mkdirSync(CREDENTIALS_DIR, { recursive: true });
  }
  
  // Generate new keypair
  const keyPair = KeyPair.fromRandom("ed25519");
  const publicKey = keyPair.getPublicKey().toString();
  
  // Save credentials
  const credFile = join(CREDENTIALS_DIR, `${ACCOUNT_ID}.json`);
  const credentials = {
    account_id: ACCOUNT_ID,
    public_key: publicKey,
    private_key: keyPair.toString()
  };
  writeFileSync(credFile, JSON.stringify(credentials, null, 2));
  console.log("Generated keypair and saved to:", credFile);
  console.log("Public key:", publicKey);
  
  // Call the faucet to create account
  console.log("\nCalling NEAR testnet faucet...");
  
  const response = await fetch("https://helper.testnet.near.org/account", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      newAccountId: ACCOUNT_ID,
      newAccountPublicKey: publicKey
    })
  });
  
  if (response.ok) {
    console.log("✅ Account created successfully!");
  } else {
    const err = await response.text();
    if (err.includes("already exists")) {
      console.log("⚠️ Account already exists. Using existing credentials.");
    } else {
      console.error("❌ Failed to create account:", err);
      process.exit(1);
    }
  }
  
  // Wait a moment for the account to be created
  await new Promise(resolve => setTimeout(resolve, 3000));
  
  console.log("\nAccount ID:", ACCOUNT_ID);
}

main().catch(console.error);

