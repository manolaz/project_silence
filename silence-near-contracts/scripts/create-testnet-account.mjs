import { KeyPair, keyStores, connect } from "near-api-js";
import { join } from "path";
import { homedir } from "os";
import { existsSync, mkdirSync, writeFileSync, readFileSync } from "fs";

const ACCOUNT_ID = "silence-bridge-kate.testnet";
const CREDENTIALS_DIR = join(homedir(), ".near-credentials", "testnet");

async function main() {
  console.log("Creating NEAR testnet account:", ACCOUNT_ID);
  
  // Ensure credentials directory exists
  if (!existsSync(CREDENTIALS_DIR)) {
    mkdirSync(CREDENTIALS_DIR, { recursive: true });
    console.log("Created credentials directory:", CREDENTIALS_DIR);
  }
  
  // Check if account already has credentials
  const credFile = join(CREDENTIALS_DIR, `${ACCOUNT_ID}.json`);
  let keyPair;
  let publicKey;
  
  if (existsSync(credFile)) {
    console.log("Credentials already exist at:", credFile);
    const creds = JSON.parse(readFileSync(credFile, "utf-8"));
    publicKey = creds.public_key;
    console.log("Using existing public key:", publicKey);
  } else {
    // Generate new keypair
    keyPair = KeyPair.fromRandom("ed25519");
    publicKey = keyPair.getPublicKey().toString();
    
    // Save credentials
    const credentials = {
      account_id: ACCOUNT_ID,
      public_key: publicKey,
      private_key: keyPair.toString()
    };
    writeFileSync(credFile, JSON.stringify(credentials, null, 2));
    console.log("Generated keypair and saved to:", credFile);
    console.log("Public key:", publicKey);
  }
  
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
    console.log("Account ID:", ACCOUNT_ID);
    console.log("Credentials saved to:", credFile);
  } else {
    const err = await response.text();
    if (err.includes("already exists")) {
      console.log("⚠️ Account already exists. Using existing credentials.");
    } else {
      console.error("❌ Failed to create account:", err);
      process.exit(1);
    }
  }
  
  // Verify the account
  console.log("\nVerifying account on testnet...");
  const config = {
    networkId: "testnet",
    nodeUrl: "https://rpc.testnet.near.org",
  };
  
  const near = await connect(config);
  try {
    const account = await near.account(ACCOUNT_ID);
    const balance = await account.getAccountBalance();
    console.log("✅ Account verified!");
    console.log("Balance:", (parseFloat(balance.available) / 1e24).toFixed(4), "NEAR");
  } catch (e) {
    console.error("Account verification failed:", e.message);
  }
}

main().catch(console.error);

