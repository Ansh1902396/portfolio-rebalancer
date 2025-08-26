import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Rebalancer } from "../target/types/rebalancer";
import { expect } from "chai";

describe("rebalancer", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Rebalancer as Program<Rebalancer>;
  const manager = anchor.web3.Keypair.generate();

  before(async () => {
    // Airdrop SOL to manager for testing
    await provider.connection.requestAirdrop(manager.publicKey, 5 * anchor.web3.LAMPORTS_PER_SOL);
    await new Promise(resolve => setTimeout(resolve, 1000)); // Wait for airdrop
  });

  it("Initializes portfolio successfully", async () => {
    const [portfolioPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("portfolio"), manager.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .initializePortfolio(
        manager.publicKey,
        25, // 25% rebalance threshold
        new anchor.BN(3600) // 1 hour minimum interval
      )
      .accountsPartial({
        portfolio: portfolioPda,
        payer: provider.wallet.publicKey,
        manager: manager.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const portfolio = await program.account.portfolio.fetch(portfolioPda);
    expect(portfolio.manager.toString()).to.equal(manager.publicKey.toString());
    expect(portfolio.rebalanceThreshold).to.equal(25);
    expect(portfolio.totalStrategies).to.equal(0);
  });

  it("Registers strategy successfully", async () => {
    const strategyId = anchor.web3.Keypair.generate().publicKey;
    const [portfolioPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("portfolio"), manager.publicKey.toBuffer()],
      program.programId
    );
    const [strategyPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("strategy"), portfolioPda.toBuffer(), strategyId.toBuffer()],
      program.programId
    );

    const protocolType = {
      stableLending: {
        poolId: anchor.web3.Keypair.generate().publicKey,
        utilization: 7500, // 75% utilization
        reserveAddress: anchor.web3.Keypair.generate().publicKey,
      }
    };

    await program.methods
      .registerStrategy(
        strategyId,
        protocolType,
        new anchor.BN(1000000000) // 1 SOL initial balance
      )
      .accountsPartial({
        portfolio: portfolioPda,
        strategy: strategyPda,
        manager: manager.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([manager])
      .rpc();

    const strategy = await program.account.strategy.fetch(strategyPda);
    expect(strategy.strategyId.toString()).to.equal(strategyId.toString());
    expect(strategy.currentBalance.toString()).to.equal("1000000000");
    expect(strategy.status).to.deep.equal({ active: {} });

    const updatedPortfolio = await program.account.portfolio.fetch(portfolioPda);
    expect(updatedPortfolio.totalStrategies).to.equal(1);
  });

  it("Validates protocol types correctly", async () => {
    const strategyId2 = anchor.web3.Keypair.generate().publicKey; // Use unique strategy ID
    const [portfolioPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("portfolio"), manager.publicKey.toBuffer()],
      program.programId
    );
    const [strategyPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("strategy"), portfolioPda.toBuffer(), strategyId2.toBuffer()],
      program.programId
    );

    // Test StableLending protocol type (simpler structure)
    const stableLendingProtocol = {
      stableLending: {
        poolId: anchor.web3.Keypair.generate().publicKey,
        utilization: 7500, // 75% utilization
        reserveAddress: anchor.web3.Keypair.generate().publicKey,
      }
    };

    await program.methods
      .registerStrategy(
        strategyId2,
        stableLendingProtocol,
        new anchor.BN(1000000000) // 1 SOL initial balance (meets 0.1 SOL minimum)
      )
      .accountsPartial({
        portfolio: portfolioPda,
        strategy: strategyPda,
        manager: manager.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([manager])
      .rpc();

    const strategy = await program.account.strategy.fetch(strategyPda);
    expect(strategy.protocolType.stableLending.utilization).to.equal(7500);
  });

  it("Tests protocol validation and balance constraints", async () => {
    // Test minimum balance constraint for StableLending (0.1 SOL minimum)
    const strategyId3 = anchor.web3.Keypair.generate().publicKey;
    const [portfolioPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("portfolio"), manager.publicKey.toBuffer()],
      program.programId
    );
    const [strategyPda3] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("strategy"), portfolioPda.toBuffer(), strategyId3.toBuffer()],
      program.programId
    );

    const protocolMinBalance = {
      stableLending: {
        poolId: anchor.web3.Keypair.generate().publicKey,
        utilization: 5000, // 50% utilization
        reserveAddress: anchor.web3.Keypair.generate().publicKey,
      }
    };

    // Test with exactly minimum balance (0.1 SOL = 100,000,000 lamports)
    await program.methods
      .registerStrategy(
        strategyId3,
        protocolMinBalance,
        new anchor.BN(100000000) // Exactly 0.1 SOL minimum
      )
      .accountsPartial({
        portfolio: portfolioPda,
        strategy: strategyPda3,
        manager: manager.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([manager])
      .rpc();

    // Verify the strategy was created with minimum balance
    const strategy = await program.account.strategy.fetch(strategyPda3);
    expect(strategy.currentBalance.toString()).to.equal("100000000");
    expect(strategy.protocolType.stableLending.utilization).to.equal(5000);

    // Verify total strategies increased
    const portfolio = await program.account.portfolio.fetch(portfolioPda);
    expect(portfolio.totalStrategies).to.equal(3); // 1 + 1 + 1
  });

  it("Prevents invalid strategy registration", async () => {
    const strategyId = anchor.web3.Keypair.generate().publicKey;
    const [portfolioPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("portfolio"), manager.publicKey.toBuffer()],
      program.programId
    );
    const [strategyPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("strategy"), portfolioPda.toBuffer(), strategyId.toBuffer()],
      program.programId
    );

    // Test invalid protocol with default pubkey
    const invalidProtocol = {
      stableLending: {
        poolId: anchor.web3.PublicKey.default,
        utilization: 5000,
        reserveAddress: anchor.web3.Keypair.generate().publicKey,
      }
    };

    try {
      await program.methods
        .registerStrategy(
          strategyId,
          invalidProtocol,
          new anchor.BN(1000000000)
        )
        .accountsPartial({
          portfolio: portfolioPda,
          strategy: strategyPda,
          manager: manager.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([manager])
        .rpc();
      
      expect.fail("Should have failed with invalid pool ID");
    } catch (error) {
      expect(error.message).to.include("InvalidProtocolType");
    }
  });
});
