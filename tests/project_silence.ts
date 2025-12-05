import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { ProjectSilence } from "../target/types/project_silence";
import { randomBytes } from "crypto";
import {
  awaitComputationFinalization,
  getArciumEnv,
  getCompDefAccOffset,
  getArciumAccountBaseSeed,
  getArciumProgAddress,
  uploadCircuit,
  buildFinalizeCompDefTx,
  RescueCipher,
  deserializeLE,
  getMXEPublicKey,
  getMXEAccAddress,
  getMempoolAccAddress,
  getCompDefAccAddress,
  getExecutingPoolAccAddress,
  getComputationAccAddress,
  x25519,
} from "@arcium-hq/client";
import * as fs from "fs";
import * as os from "os";
import { expect } from "chai";

describe("ProjectSilence", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.ProjectSilence as Program<ProjectSilence>;
  const provider = anchor.getProvider() as anchor.AnchorProvider;

  type Event = anchor.IdlEvents<(typeof program)["idl"]>;
  const awaitEvent = async <E extends keyof Event>(
    eventName: E
  ): Promise<Event[E]> => {
    let listenerId: number;
    const event = await new Promise<Event[E]>((res) => {
      listenerId = program.addEventListener(eventName, (event) => {
        res(event);
      });
    });
    await program.removeEventListener(listenerId);
    return event;
  };

  const arciumEnv = getArciumEnv();
  let owner: Keypair;
  let configPda: PublicKey;
  let feeVault: Keypair;

  before(async () => {
    owner = readKpJson(`${os.homedir()}/.config/solana/id.json`);
    feeVault = Keypair.generate();
    
    // Find config PDA
    [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bridge_config")],
      program.programId
    );
  });

  // =========================================================================
  // BRIDGE INITIALIZATION TESTS
  // =========================================================================

  describe("Bridge Initialization", () => {
    it("initializes bridge configuration", async () => {
      const minSolverStake = new anchor.BN(LAMPORTS_PER_SOL); // 1 SOL
      const protocolFeeBps = 50; // 0.5%

      await program.methods
        .initializeBridge(minSolverStake, protocolFeeBps)
        .accounts({
          owner: owner.publicKey,
          config: configPda,
          feeVault: feeVault.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      const config = await program.account.bridgeConfig.fetch(configPda);
      expect(config.owner.toString()).to.equal(owner.publicKey.toString());
      expect(config.minSolverStake.toNumber()).to.equal(minSolverStake.toNumber());
      expect(config.protocolFeeBps).to.equal(protocolFeeBps);
    });
  });

  // =========================================================================
  // MODEL REGISTRY TESTS
  // =========================================================================

  describe("Model Registry", () => {
    const modelId = new anchor.BN(1);
    let modelPda: PublicKey;

    before(() => {
      [modelPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("model"), modelId.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
    });

    it("registers a new AI model", async () => {
      const name = "GPT-4 Agent";
      const description = "AI model for DeFi strategy analysis";
      const modelType = 0; // LLM
      const version = "1.0.0";
      const teeRequired = true;
      const attestationRequired = true;
      const costPerInference = new anchor.BN(100000); // 0.0001 SOL

      await program.methods
        .registerModel(
          modelId,
          name,
          description,
          modelType,
          version,
          teeRequired,
          attestationRequired,
          costPerInference
        )
        .accounts({
          owner: owner.publicKey,
          model: modelPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      const model = await program.account.modelMetadata.fetch(modelPda);
      expect(model.name).to.equal(name);
      expect(model.modelType).to.equal(modelType);
      expect(model.teeRequired).to.equal(teeRequired);
      expect(model.isActive).to.equal(true);
    });

    it("updates model metadata", async () => {
      const newVersion = "1.1.0";
      const newCost = new anchor.BN(150000);

      await program.methods
        .updateModel(
          null, // name unchanged
          null, // description unchanged
          newVersion,
          newCost,
          null // is_active unchanged
        )
        .accounts({
          owner: owner.publicKey,
          model: modelPda,
        })
        .signers([owner])
        .rpc();

      const model = await program.account.modelMetadata.fetch(modelPda);
      expect(model.version).to.equal(newVersion);
      expect(model.costPerInference.toNumber()).to.equal(newCost.toNumber());
    });

    it("creates an inference request", async () => {
      const requestId = new anchor.BN(1);
      const promptHash = Array.from(randomBytes(32));
      const requireAttestation = true;

      const [requestPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("request"), requestId.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      const [userMetricsPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_metrics"), owner.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .createInferenceRequest(requestId, promptHash, requireAttestation)
        .accounts({
          user: owner.publicKey,
          model: modelPda,
          modelOwner: owner.publicKey,
          request: requestPda,
          userMetrics: userMetricsPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      const request = await program.account.inferenceRequest.fetch(requestPda);
      expect(request.requestId.toNumber()).to.equal(requestId.toNumber());
      expect(request.status).to.equal(0); // Pending
      expect(request.requireAttestation).to.equal(true);

      const metrics = await program.account.userMetrics.fetch(userMetricsPda);
      expect(metrics.totalInferences.toNumber()).to.equal(1);
    });
  });

  // =========================================================================
  // BATCH INFERENCE TESTS
  // =========================================================================

  describe("Batch Inference", () => {
    const modelId = new anchor.BN(1);
    const batchId = new anchor.BN(1);
    let modelPda: PublicKey;
    let batchPda: PublicKey;

    before(() => {
      [modelPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("model"), modelId.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      [batchPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("batch"), batchId.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
    });

    it("creates a batch inference request", async () => {
      const promptCount = 10;
      const requireAttestation = true;

      const [userMetricsPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_metrics"), owner.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .createBatchInference(batchId, promptCount, requireAttestation)
        .accounts({
          user: owner.publicKey,
          model: modelPda,
          modelOwner: owner.publicKey,
          batch: batchPda,
          userMetrics: userMetricsPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      const batch = await program.account.batchInference.fetch(batchPda);
      expect(batch.batchId.toNumber()).to.equal(batchId.toNumber());
      expect(batch.promptCount).to.equal(promptCount);
      expect(batch.completedCount).to.equal(0);
    });
  });

  // =========================================================================
  // SOLVER REGISTRATION TESTS
  // =========================================================================

  describe("Solver Registration", () => {
    let solver: Keypair;
    let solverPda: PublicKey;

    before(async () => {
      solver = Keypair.generate();
      
      // Airdrop SOL to solver for stake
      const sig = await provider.connection.requestAirdrop(
        solver.publicKey,
        2 * LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(sig);

      [solverPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("solver"), solver.publicKey.toBuffer()],
        program.programId
      );
    });

    it("registers a new solver", async () => {
      const supportedChains = 0b011; // Solana + NEAR

      await program.methods
        .registerSolver(supportedChains)
        .accounts({
          user: solver.publicKey,
          config: configPda,
          solver: solverPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([solver])
        .rpc();

      const solverAccount = await program.account.solver.fetch(solverPda);
      expect(solverAccount.solverId.toString()).to.equal(solver.publicKey.toString());
      expect(solverAccount.supportedChains).to.equal(supportedChains);
      expect(solverAccount.reputationScore).to.equal(100);
      expect(solverAccount.isActive).to.equal(true);
    });
  });

  // =========================================================================
  // INTENT LIFECYCLE TESTS
  // =========================================================================

  describe("Intent Lifecycle", () => {
    let creator: Keypair;
    let solver: Keypair;
    let intentPda: PublicKey;
    let solverPda: PublicKey;
    const intentId = new anchor.BN(1);

    before(async () => {
      creator = Keypair.generate();
      solver = Keypair.generate();

      // Airdrop SOL
      const sig1 = await provider.connection.requestAirdrop(
        creator.publicKey,
        5 * LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(sig1);

      const sig2 = await provider.connection.requestAirdrop(
        solver.publicKey,
        2 * LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(sig2);

      [intentPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("intent"), intentId.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      [solverPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("solver"), solver.publicKey.toBuffer()],
        program.programId
      );

      // Register solver first
      await program.methods
        .registerSolver(0b111) // All chains
        .accounts({
          user: solver.publicKey,
          config: configPda,
          solver: solverPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([solver])
        .rpc();
    });

    it("creates a cross-chain intent", async () => {
      const destinationChain = { near: {} };
      const destinationAmountCommitment = Array.from(randomBytes(32));
      const destinationTokenHash = Array.from(randomBytes(32));
      const recipientHash = Array.from(randomBytes(32));
      const isShielded = false;
      const ttlSeconds = new anchor.BN(3600);
      const sourceAmount = new anchor.BN(LAMPORTS_PER_SOL);

      // Derive intent vault PDA
      const [intentVaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("intent_vault"), intentId.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      await program.methods
        .createIntent(
          intentId,
          destinationChain,
          destinationAmountCommitment,
          destinationTokenHash,
          recipientHash,
          isShielded,
          ttlSeconds,
          sourceAmount
        )
        .accounts({
          creator: creator.publicKey,
          config: configPda,
          intent: intentPda,
          intentVault: intentVaultPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([creator])
        .rpc();

      const intent = await program.account.intent.fetch(intentPda);
      expect(intent.intentId.toNumber()).to.equal(intentId.toNumber());
      expect(intent.creator.toString()).to.equal(creator.publicKey.toString());
      expect(intent.status).to.deep.equal({ created: {} });
      expect(intent.isShielded).to.equal(isShielded);
      expect(intent.sourceAmount.toNumber()).to.equal(sourceAmount.toNumber());

      // Verify funds are escrowed in intent vault
      const vaultBalance = await provider.connection.getBalance(intentVaultPda);
      expect(vaultBalance).to.equal(sourceAmount.toNumber());
    });

    it("matches an intent with a solver", async () => {
      await program.methods
        .matchIntent()
        .accounts({
          solverAuthority: solver.publicKey,
          intent: intentPda,
          solver: solverPda,
        })
        .signers([solver])
        .rpc();

      const intent = await program.account.intent.fetch(intentPda);
      expect(intent.status).to.deep.equal({ matched: {} });
      expect(intent.solver.toString()).to.equal(solver.publicKey.toString());
    });

    it("executes an intent", async () => {
      const destinationTxHash = Array.from(randomBytes(32));

      await program.methods
        .executeIntent(destinationTxHash, null)
        .accounts({
          solverAuthority: solver.publicKey,
          intent: intentPda,
        })
        .signers([solver])
        .rpc();

      const intent = await program.account.intent.fetch(intentPda);
      expect(intent.status).to.deep.equal({ executed: {} });
      expect(intent.executedAt).to.not.be.null;
    });
  });

  // =========================================================================
  // ENCRYPTED COMPUTATION TESTS (Arcium)
  // =========================================================================

  describe("Encrypted Computations", () => {
    it("initializes process inference computation definition", async () => {
      const baseSeedCompDefAcc = getArciumAccountBaseSeed(
        "ComputationDefinitionAccount"
      );
      const offset = getCompDefAccOffset("process_inference");

      const compDefPDA = PublicKey.findProgramAddressSync(
        [baseSeedCompDefAcc, program.programId.toBuffer(), offset],
        getArciumProgAddress()
      )[0];

      const sig = await program.methods
        .initProcessInferenceCompDef()
        .accounts({
          compDefAccount: compDefPDA,
          payer: owner.publicKey,
          mxeAccount: getMXEAccAddress(program.programId),
        })
        .signers([owner])
        .rpc({ commitment: "confirmed" });

      console.log("Init process inference comp def signature:", sig);

      // Finalize computation definition
      const finalizeTx = await buildFinalizeCompDefTx(
        provider,
        Buffer.from(offset).readUInt32LE(),
        program.programId
      );

      const latestBlockhash = await provider.connection.getLatestBlockhash();
      finalizeTx.recentBlockhash = latestBlockhash.blockhash;
      finalizeTx.lastValidBlockHeight = latestBlockhash.lastValidBlockHeight;
      finalizeTx.sign(owner);

      await provider.sendAndConfirm(finalizeTx);
    });

    it("initializes verify intent amounts computation definition", async () => {
      const baseSeedCompDefAcc = getArciumAccountBaseSeed(
        "ComputationDefinitionAccount"
      );
      const offset = getCompDefAccOffset("verify_intent_amounts");

      const compDefPDA = PublicKey.findProgramAddressSync(
        [baseSeedCompDefAcc, program.programId.toBuffer(), offset],
        getArciumProgAddress()
      )[0];

      const sig = await program.methods
        .initVerifyIntentAmountsCompDef()
        .accounts({
          compDefAccount: compDefPDA,
          payer: owner.publicKey,
          mxeAccount: getMXEAccAddress(program.programId),
        })
        .signers([owner])
        .rpc({ commitment: "confirmed" });

      console.log("Init verify intent amounts comp def signature:", sig);

      // Finalize computation definition
      const finalizeTx = await buildFinalizeCompDefTx(
        provider,
        Buffer.from(offset).readUInt32LE(),
        program.programId
      );

      const latestBlockhash = await provider.connection.getLatestBlockhash();
      finalizeTx.recentBlockhash = latestBlockhash.blockhash;
      finalizeTx.lastValidBlockHeight = latestBlockhash.lastValidBlockHeight;
      finalizeTx.sign(owner);

      await provider.sendAndConfirm(finalizeTx);
    });

    it("initializes generate privacy proof computation definition", async () => {
      const baseSeedCompDefAcc = getArciumAccountBaseSeed(
        "ComputationDefinitionAccount"
      );
      const offset = getCompDefAccOffset("generate_privacy_proof");

      const compDefPDA = PublicKey.findProgramAddressSync(
        [baseSeedCompDefAcc, program.programId.toBuffer(), offset],
        getArciumProgAddress()
      )[0];

      const sig = await program.methods
        .initGeneratePrivacyProofCompDef()
        .accounts({
          compDefAccount: compDefPDA,
          payer: owner.publicKey,
          mxeAccount: getMXEAccAddress(program.programId),
        })
        .signers([owner])
        .rpc({ commitment: "confirmed" });

      console.log("Init generate privacy proof comp def signature:", sig);

      // Finalize computation definition
      const finalizeTx = await buildFinalizeCompDefTx(
        provider,
        Buffer.from(offset).readUInt32LE(),
        program.programId
      );

      const latestBlockhash = await provider.connection.getLatestBlockhash();
      finalizeTx.recentBlockhash = latestBlockhash.blockhash;
      finalizeTx.lastValidBlockHeight = latestBlockhash.lastValidBlockHeight;
      finalizeTx.sign(owner);

      await provider.sendAndConfirm(finalizeTx);
    });

    it("queues and processes an encrypted inference", async () => {
      const mxePublicKey = await getMXEPublicKeyWithRetry(
        provider,
        program.programId
      );

      const privateKey = x25519.utils.randomSecretKey();
      const publicKey = x25519.getPublicKey(privateKey);
      const sharedSecret = x25519.getSharedSecret(privateKey, mxePublicKey);
      const cipher = new RescueCipher(sharedSecret);

      // Create encrypted inference input
      const promptHash = BigInt("0x" + randomBytes(32).toString("hex"));
      const modelId = BigInt(1);
      const nonce = BigInt(Date.now());

      const plaintext = [promptHash, modelId, nonce];
      const encNonce = randomBytes(16);
      const ciphertext = cipher.encrypt(plaintext, encNonce);

      const attestationKey = Array.from(randomBytes(32));

      const inferenceProcessedEventPromise = awaitEvent("inferenceProcessed");
      const computationOffset = new anchor.BN(randomBytes(8), "hex");

      const sig = await program.methods
        .processInference(
          computationOffset,
          Array.from(ciphertext[0]),
          Array.from(ciphertext[1]),
          Array.from(ciphertext[2]),
          attestationKey,
          Array.from(publicKey),
          new anchor.BN(deserializeLE(encNonce).toString())
        )
        .accountsPartial({
          computationAccount: getComputationAccAddress(
            program.programId,
            computationOffset
          ),
          clusterAccount: arciumEnv.arciumClusterPubkey,
          mxeAccount: getMXEAccAddress(program.programId),
          mempoolAccount: getMempoolAccAddress(program.programId),
          executingPool: getExecutingPoolAccAddress(program.programId),
          compDefAccount: getCompDefAccAddress(
            program.programId,
            Buffer.from(getCompDefAccOffset("process_inference")).readUInt32LE()
          ),
        })
        .rpc({ skipPreflight: true, commitment: "confirmed" });

      console.log("Process inference queue signature:", sig);

      const finalizeSig = await awaitComputationFinalization(
        provider,
        computationOffset,
        program.programId,
        "confirmed"
      );
      console.log("Process inference finalize signature:", finalizeSig);

      const event = await inferenceProcessedEventPromise;
      console.log("Inference processed event received");
      expect(event.resultHash).to.have.lengthOf(32);
    });
  });

  // =========================================================================
  // ADMIN TESTS
  // =========================================================================

  describe("Admin Functions", () => {
    it("updates protocol fee", async () => {
      const newFeeBps = 100; // 1%

      await program.methods
        .setProtocolFee(newFeeBps)
        .accounts({
          owner: owner.publicKey,
          config: configPda,
        })
        .signers([owner])
        .rpc();

      const config = await program.account.bridgeConfig.fetch(configPda);
      expect(config.protocolFeeBps).to.equal(newFeeBps);
    });

    it("rejects fee above maximum", async () => {
      const tooHighFee = 1100; // 11%

      try {
        await program.methods
          .setProtocolFee(tooHighFee)
          .accounts({
            owner: owner.publicKey,
            config: configPda,
          })
          .signers([owner])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err) {
        expect(err.message).to.include("FeeTooHigh");
      }
    });
  });
});

// =========================================================================
// HELPER FUNCTIONS
// =========================================================================

async function getMXEPublicKeyWithRetry(
  provider: anchor.AnchorProvider,
  programId: PublicKey,
  maxRetries: number = 10,
  retryDelayMs: number = 500
): Promise<Uint8Array> {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      const mxePublicKey = await getMXEPublicKey(provider, programId);
      if (mxePublicKey) {
        return mxePublicKey;
      }
    } catch (error) {
      console.log(`Attempt ${attempt} failed to fetch MXE public key:`, error);
    }

    if (attempt < maxRetries) {
      console.log(
        `Retrying in ${retryDelayMs}ms... (attempt ${attempt}/${maxRetries})`
      );
      await new Promise((resolve) => setTimeout(resolve, retryDelayMs));
    }
  }

  throw new Error(
    `Failed to fetch MXE public key after ${maxRetries} attempts`
  );
}

function readKpJson(path: string): anchor.web3.Keypair {
  const file = fs.readFileSync(path);
  return anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(file.toString()))
  );
}
