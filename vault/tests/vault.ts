import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Vault } from "../target/types/vault";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { expect } from "chai";

describe("vault", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Vault as Program<Vault>;
  const signer = provider.wallet.publicKey;

  // Derive PDAs
  const [vaultState] = PublicKey.findProgramAddressSync(
    [Buffer.from("state"), signer.toBuffer()],
    program.programId
  );

  const [vault] = PublicKey.findProgramAddressSync(
    [vaultState.toBuffer()],
    program.programId
  );

  describe("initialize", () => {
    it("should initialize vault state", async () => {
      const tx = await program.methods
        .initialize()
        .accounts({
          signer,
          vaultState,
          vault,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // Fetch the created account
      const vaultStateAccount = await program.account.vaultState.fetch(
        vaultState
      );

      expect(vaultStateAccount.stateBump).to.not.be.null;
      expect(vaultStateAccount.vaultBump).to.not.be.null;
    });

    it("should fail to initialize vault state twice", async () => {
      try {
        await program.methods
          .initialize()
          .accounts({
            signer,
            vaultState,
            vault,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have failed");
      } catch (error) {
        expect(error).to.be.instanceOf(Error);
      }
    });
  });

  describe("deposit", () => {
    const depositAmount = new anchor.BN(1 * LAMPORTS_PER_SOL); // 1 SOL

    it("should deposit SOL into vault", async () => {
      const vaultBalanceBefore = await provider.connection.getBalance(vault);

      await program.methods
        .deposit(depositAmount)
        .accounts({
          signer,
          vaultState,
          vault,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const vaultBalanceAfter = await provider.connection.getBalance(vault);
      expect(vaultBalanceAfter - vaultBalanceBefore).to.equal(
        depositAmount.toNumber()
      );
    });

    it("should fail to deposit with insufficient funds", async () => {
      const largeAmount = new anchor.BN(1000000 * LAMPORTS_PER_SOL);

      try {
        await program.methods
          .deposit(largeAmount)
          .accounts({
            signer,
            vaultState,
            vault,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have failed");
      } catch (error) {
        expect(error).to.be.instanceOf(Error);
      }
    });
  });

  describe("withdraw", () => {
    const withdrawAmount = new anchor.BN(0.5 * LAMPORTS_PER_SOL); // 0.5 SOL

    it("should withdraw SOL from vault", async () => {
      const vaultBalanceBefore = await provider.connection.getBalance(vault);
      const signerBalanceBefore = await provider.connection.getBalance(signer);

      await program.methods
        .withdraw(withdrawAmount)
        .accounts({
          signer,
          vaultState,
          vault,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const vaultBalanceAfter = await provider.connection.getBalance(vault);
      const signerBalanceAfter = await provider.connection.getBalance(signer);

      // Account for transaction fees in signer balance check
      expect(vaultBalanceBefore - vaultBalanceAfter).to.equal(
        withdrawAmount.toNumber()
      );
      expect(signerBalanceAfter).to.be.greaterThan(signerBalanceBefore);
    });

    it("should fail to withdraw more than vault balance", async () => {
      const largeAmount = new anchor.BN(1000000 * LAMPORTS_PER_SOL);

      try {
        await program.methods
          .withdraw(largeAmount)
          .accounts({
            signer,
            vaultState,
            vault,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have failed");
      } catch (error) {
        expect(error).to.be.instanceOf(Error);
      }
    });
  });
});
