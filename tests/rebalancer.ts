import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Rebalancer } from "../target/types/rebalancer";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";

describe("rebalancer", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.rebalancer as Program<Rebalancer>;
  
  // Test accounts
  const manager = Keypair.generate();
  const authority = provider.wallet as anchor.Wallet;
  
  let portfolioPda: PublicKey;
  let portfolioBump: number;
  
  before(async () => {
    // Derive portfolio PDA
    [portfolioPda, portfolioBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("portfolio"), manager.publicKey.toBuffer()],
      program.programId
    );
    
    // Airdrop SOL to manager for testing
    await provider.connection.requestAirdrop(manager.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(authority.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
  });

  it("Legacy initialize works", async () => {
    const tx = await program.methods.initialize().rpc();
    console.log("Legacy initialize transaction signature:", tx);
  });

  it("Initialize portfolio successfully", async () => {
    const rebalanceThreshold = 5; // 5%
    const minRebalanceInterval = new BN(3600); // 1 hour
    
    const tx = await program.methods
      .initializePortfolio(
        manager.publicKey,
        rebalanceThreshold,
        minRebalanceInterval
      )
      .accountsPartial({
        portfolio: portfolioPda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log("Initialize portfolio transaction signature:", tx);
    
    // Verify portfolio was created correctly
    const portfolioAccount = await program.account.portfolio.fetch(portfolioPda);
    
    assert.equal(portfolioAccount.manager.toString(), manager.publicKey.toString());
    assert.equal(portfolioAccount.rebalanceThreshold, rebalanceThreshold);
    assert.equal(portfolioAccount.minRebalanceInterval.toString(), minRebalanceInterval.toString());
    assert.equal(portfolioAccount.strategyCount, 0);
    assert.equal(portfolioAccount.totalValue.toString(), "0");
    assert.equal(portfolioAccount.bump, portfolioBump);
  });

  it("Register strategy successfully", async () => {
    const strategyId = Keypair.generate().publicKey;
    const protocolType = { lending: {} }; // Enum variant for Lending
    const initialBalance = new BN(1000000); // 1M tokens
    
    // Derive strategy PDA
    const [strategyPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("strategy"), portfolioPda.toBuffer(), strategyId.toBuffer()],
      program.programId
    );
    
    const tx = await program.methods
      .registerStrategy(
        strategyId,
        protocolType,
        initialBalance
      )
      .accountsPartial({
        portfolio: portfolioPda,
        strategy: strategyPda,
        authority: manager.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([manager])
      .rpc();
    
    console.log("Register strategy transaction signature:", tx);
    
    // Verify strategy was created correctly
    const strategyAccount = await program.account.strategy.fetch(strategyPda);
    
    assert.equal(strategyAccount.portfolio.toString(), portfolioPda.toString());
    assert.equal(strategyAccount.strategyId.toString(), strategyId.toString());
    assert.deepEqual(strategyAccount.protocolType, protocolType);
    assert.equal(strategyAccount.currentBalance.toString(), initialBalance.toString());
    assert.equal(strategyAccount.targetAllocation, 0);
    assert.equal(strategyAccount.isActive, true);
    
    // Verify portfolio was updated
    const portfolioAccount = await program.account.portfolio.fetch(portfolioPda);
    assert.equal(portfolioAccount.strategyCount, 1);
    assert.equal(portfolioAccount.totalValue.toString(), initialBalance.toString());
  });

  it("Fails to register strategy with unauthorized manager", async () => {
    const unauthorized = Keypair.generate();
    const strategyId = Keypair.generate().publicKey;
    const protocolType = { dex: {} };
    const initialBalance = new BN(500000);
    
    // Airdrop to unauthorized account and wait for confirmation
    const airdropTx = await provider.connection.requestAirdrop(unauthorized.publicKey, anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(airdropTx, "confirmed");
    
    const [strategyPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("strategy"), portfolioPda.toBuffer(), strategyId.toBuffer()],
      program.programId
    );
    
    try {
      await program.methods
        .registerStrategy(
          strategyId,
          protocolType,
          initialBalance
        )
        .accountsPartial({
          portfolio: portfolioPda,
          strategy: strategyPda,
          authority: unauthorized.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([unauthorized])
        .rpc();
      
      assert.fail("Should have failed with unauthorized access");
    } catch (error) {
      // Check for constraint error or access control error
      assert(error.message.includes("A raw constraint was violated") || 
             error.message.includes("InvalidManager") ||
             error.message.includes("constraint"));
    }
  });

  it("Fails to initialize portfolio with invalid parameters", async () => {
    const invalidManager = Keypair.generate();
    const [invalidPortfolioPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("portfolio"), invalidManager.publicKey.toBuffer()],
      program.programId
    );
    
    try {
      await program.methods
        .initializePortfolio(
          invalidManager.publicKey,
          101, // Invalid threshold > 100
          new BN(3600)
        )
        .accountsPartial({
          portfolio: invalidPortfolioPda,
          authority: authority.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      
      assert.fail("Should have failed with invalid threshold");
    } catch (error) {
      assert(error.message.includes("InvalidRebalanceThreshold"));
    }
  });
});
