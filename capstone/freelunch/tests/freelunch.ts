import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import { assert } from "chai";
import {
  createMint,
  createAccount,
  mintTo,
  getAccount,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createInitializeAccountInstruction,
  getMinimumBalanceForRentExemptAccount,
} from "@solana/spl-token";
import { Freelunch } from "../target/types/freelunch";

/**
 * Example test suite matching your IDL, now with a separate `protocol_usdc_account`.
 *
 * IDL references:
 *  init => protocol_vault, admin, system_program
 *  stake => buyer, buyer_usdc_account, buyer_account, protocol_vault, ...
 *  merchant_init => merchant, merchant_account, system_program
 *  create_proof_of_payment => admin, buyer_account, proof_of_payment, merchant_account, merchant, solend_reserve, system_program
 *  fulfill_proof_of_payment => protocol_signer, protocol_vault, protocol_usdc_account, merchant_usdc_account, ...
 *  merchant_claim => merchant, proof_of_payment, buyer_account, protocol_usdc_account, merchant_usdc_account, ...
 *  unstake => buyer, buyer_account, protocol_vault, protocol_usdc_account, buyer_usdc_account, ...
 */

describe("freelunch", () => {
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);
  const connection = provider.connection;

  const program = anchor.workspace.Freelunch as Program<Freelunch>;

  // Keypairs for testing
  const admin = Keypair.generate();
  const buyer = Keypair.generate();
  const merchant = Keypair.generate();

  // We'll create a USDC mint and various token accounts
  let usdcMint: PublicKey;
  let buyerUsdcAccount: PublicKey;
  let merchantUsdcAccount: PublicKey;
  // let protocolUsdcAccount: PublicKey;
  let protocolCollateralAccount: PublicKey;

  // PDAs
  let protocolVaultPda: PublicKey;
  let protocolVaultBump: number;

  let buyerAccountPda: PublicKey;
  let buyerAccountBump: number;

  let merchantAccountPda: PublicKey;
  let merchantAccountBump: number;

  let proofOfPaymentPda: PublicKey;
  let proofOfPaymentBump: number;

  let solendProgram: PublicKey;
  let solendReserve: PublicKey;
  let reserveLiquiditySupply: PublicKey;
  let reserveCollateralMint: PublicKey;
  let lendingMarket: PublicKey;
  let lendingMarketAuthority: PublicKey;

  // Airdrop convenience
  const airdrop = async (pk: PublicKey, solAmount = 2) => {
    const sig = await connection.requestAirdrop(
      pk,
      solAmount * anchor.web3.LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(sig);
  };

  before(async () => {
    await airdrop(admin.publicKey, 5);
    await airdrop(buyer.publicKey, 5);
    await airdrop(merchant.publicKey, 5);

    usdcMint = await createMint(connection, admin, admin.publicKey, null, 6);

    console.log("USDC mint:", usdcMint.toBase58());

    // 3) Create buyer + merchant token accounts
    buyerUsdcAccount = await createAccount(
      connection,
      admin,
      usdcMint,
      buyer.publicKey
    );

    console.log("Buyer USDC account:", buyerUsdcAccount.toBase58());

    merchantUsdcAccount = await createAccount(
      connection,
      admin,
      usdcMint,
      merchant.publicKey
    );

    console.log("Merchant USDC account:", merchantUsdcAccount.toBase58());

    await mintTo(
      connection,
      admin,
      usdcMint,
      buyerUsdcAccount,
      admin.publicKey,
      1_000_000_000
    );

    reserveCollateralMint = await createMint(
      connection,
      admin,
      admin.publicKey,
      null,
      6
    );

    console.log("Reserve collateral mint:", reserveCollateralMint.toBase58());

    [protocolVaultPda, protocolVaultBump] =
      await PublicKey.findProgramAddressSync(
        [Buffer.from("protocol_vault")],
        program.programId
      );

    console.log("Protocol vault PDA:", protocolVaultPda.toBase58());

    const protocolCollateralAccountKeypair = Keypair.generate();
    const lamportsForRent = await getMinimumBalanceForRentExemptAccount(
      connection
    );

    const transaction = new Transaction();
    transaction.add(
      SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: protocolCollateralAccountKeypair.publicKey,
        space: 165, // SPL Token Account size
        lamports: lamportsForRent,
        programId: TOKEN_PROGRAM_ID,
      }),
      createInitializeAccountInstruction(
        protocolCollateralAccountKeypair.publicKey,
        reserveCollateralMint,
        protocolVaultPda,
        TOKEN_PROGRAM_ID
      )
    );

    await provider.sendAndConfirm(transaction, [
      admin,
      protocolCollateralAccountKeypair,
    ]);

    protocolCollateralAccount = protocolCollateralAccountKeypair.publicKey;

    // protocolCollateralAccount = await getAssociatedTokenAddress(
    //   reserveCollateralMint, // The mock reserve collateral mint (cUSDC)
    //   protocolVaultPda, // The protocol vault PDA as owner
    //   true // Allow off-curve PDA as owner
    // );

    console.log(
      "Protocol collateral account:",
      protocolCollateralAccount.toBase58()
    );

    [buyerAccountPda, buyerAccountBump] =
      await PublicKey.findProgramAddressSync(
        [Buffer.from("buyer"), buyer.publicKey.toBuffer()],
        program.programId
      );
    console.log("Buyer account PDA:", buyerAccountPda.toBase58());

    [merchantAccountPda, merchantAccountBump] =
      await PublicKey.findProgramAddressSync(
        [Buffer.from("merchant"), merchant.publicKey.toBuffer()],
        program.programId
      );
    console.log("Merchant account PDA:", merchantAccountPda.toBase58());

    [solendProgram] = await PublicKey.findProgramAddressSync(
      [Buffer.from("mock_solend_program")],
      program.programId
    );
    console.log("Solend program:", solendProgram.toBase58());

    [solendReserve] = await PublicKey.findProgramAddressSync(
      [Buffer.from("mock_solend_reserve")],
      program.programId
    );
    console.log("Solend reserve PDA:", solendReserve.toBase58());

    [reserveLiquiditySupply] = await PublicKey.findProgramAddressSync(
      [Buffer.from("mock_liquidity_supply")],
      program.programId
    );
    console.log(
      "Reserve liquidity supply PDA:",
      reserveLiquiditySupply.toBase58()
    );

    // [reserveCollateralMint] = await PublicKey.findProgramAddressSync(
    //   [Buffer.from("mock_collateral_mint")],
    //   program.programId
    // );
    [lendingMarket] = await PublicKey.findProgramAddressSync(
      [Buffer.from("mock_lending_market")],
      program.programId
    );
    console.log("Lending market PDA:", lendingMarket.toBase58());

    [lendingMarketAuthority] = await PublicKey.findProgramAddressSync(
      [Buffer.from("mock_lending_market_authority")],
      program.programId
    );
    console.log(
      "Lending market authority PDA:",
      lendingMarketAuthority.toBase58()
    );
  });

  it("Initialize Protocol Vault", async () => {
    // IDL: init => protocol_vault, admin, system_program
    const txSig = await program.methods
      .init()
      .accounts({
        protocol_vault: protocolVaultPda,
        admin: admin.publicKey,
        system_program: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

    console.log("init tx:", txSig);

    const vaultState = await program.account.protocolVault.fetch(
      protocolVaultPda
    );
    assert.ok(
      vaultState.admin.equals(admin.publicKey),
      "Vault admin must match"
    );
    assert.equal(vaultState.totalStaked.toNumber(), 0);
    assert.equal(vaultState.bump, protocolVaultBump);
  });

  it("Stake (Buyer)", async () => {
    // stake => buyer, buyer_usdc_account, buyer_account, protocol_vault, ...
    const stakeAmt = new anchor.BN(100_000_000);

    await program.methods
      .stake(stakeAmt)
      .accounts({
        buyer: buyer.publicKey,
        buyerUsdcAccount: buyerUsdcAccount,
        buyer_account: buyerAccountPda,
        protocol_vault: protocolVaultPda,
        solendProgram: solendProgram,
        solendReserve: solendReserve,
        reserveLiquiditySupply: reserveLiquiditySupply,
        reserveCollateralMint: reserveCollateralMint,
        lendingMarket: lendingMarket,
        lendingMarketAuthority: lendingMarketAuthority,
        protocolCollateralAccount: protocolCollateralAccount,
        token_program: TOKEN_PROGRAM_ID,
        system_program: SystemProgram.programId,
      })
      .preInstructions([
        new anchor.web3.TransactionInstruction({
          keys: [{ pubkey: solendProgram, isWritable: true, isSigner: false }],
          programId: solendProgram,
        }),
      ])
      .signers([buyer])
      .rpc();

    const buyerState = await program.account.buyerAccount.fetch(
      buyerAccountPda
    );
    assert.ok(buyerState.buyer.equals(buyer.publicKey));
    assert.equal(buyerState.stakedAmount.toNumber(), 100_000_000);
  });

  it("Merchant Init", async () => {
    // merchant_init => merchant, merchant_account, system_program
    const seedValue = new anchor.BN(1234);

    await program.methods
      .merchantInit(seedValue)
      .accounts({
        merchant: merchant.publicKey,
        merchant_account: merchantAccountPda,
        system_program: SystemProgram.programId,
      })
      .signers([merchant])
      .rpc();

    const merchantState = await program.account.merchantAccount.fetch(
      merchantAccountPda
    );
    assert.ok(merchantState.merchant.equals(merchant.publicKey));
    assert.equal(merchantState.status, 1);
    assert.equal(merchantState.paymentNumber.toNumber(), 0);
  });

  it("Purchase", async () => {
    const purchaseAmount = new anchor.BN(5_000_000);
    const bufferBps = new anchor.BN(500);

    const merchantBefore = await program.account.merchantAccount.fetch(
      merchantAccountPda
    );
    const currentPaymentNum = merchantBefore.paymentNumber;

    // Seeds: ["proof_of_payment", buyer_account.buyer, merchant_account.merchant, merchant_account.payment_number]
    const [pofPda, pofBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("proof_of_payment"),
        buyer.publicKey.toBuffer(),
        merchant.publicKey.toBuffer(),
        new anchor.BN(currentPaymentNum).toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
    proofOfPaymentPda = pofPda;
    proofOfPaymentBump = pofBump;

    await program.methods
      .createProofOfPayment(purchaseAmount, bufferBps)
      .accounts({
        admin: admin.publicKey,
        buyerAccount: buyerAccountPda,
        // proof_of_payment: proofOfPaymentPda,
        // merchant_account: merchantAccountPda,
        merchant: merchant.publicKey,
        solendReserve: solendReserve,
        // system_program: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

    const pofState = await program.account.proofOfFuturePayment.fetch(
      proofOfPaymentPda
    );
    assert.equal(pofState.paymentAmount.toNumber(), 5_000_000);
  });

  it("Fulfill Proof Of Payment (partial)", async () => {
    // fulfill_proof_of_payment => protocol_signer, protocol_vault, protocol_usdc_account, merchant_usdc_account, ...
    const payNow = new anchor.BN(3_000_000);

    await program.methods
      .fulfillProofOfPayment(payNow)
      .accounts({
        protocolSigner: admin.publicKey,
        protocol_vault: protocolVaultPda,
        protocolUsdcAccount: merchantUsdcAccount, // <--- Using protocolUsdcAccount now
        merchantUsdcAccount: merchantUsdcAccount,
        proofOfPayment: proofOfPaymentPda,
        buyerAccount: buyerAccountPda,
        merchat_account: merchantAccountPda,
        solendProgram: solendProgram,
        solend_reserve: solendReserve,
        reserveLiquiditySupply: reserveLiquiditySupply,
        reserve_collateral_mint: reserveCollateralMint,
        lendingMarket: lendingMarket,
        lendingMarketAuthority: lendingMarketAuthority,
        protocolCollateralAccount: protocolCollateralAccount,
        token_program: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();
  });

  it("Merchant Claim (partial)", async () => {
    const claimAmount = new anchor.BN(1_000_000);

    await program.methods
      .merchantClaim(claimAmount)
      .accounts({
        merchant: merchant.publicKey,
        proofOfPayment: proofOfPaymentPda,
        buyerAccount: buyerAccountPda,
        protocolUsdcAccount: merchantUsdcAccount, // <--- The protocol's USDC
        merchantUsdcAccount: merchantUsdcAccount,
        merchant_account: merchantAccountPda,
        protocol_vault: protocolVaultPda,
        token_program: TOKEN_PROGRAM_ID,
      })
      .signers([merchant])
      .rpc();
  });

  it("Fulfill final portion (complete PoF)", async () => {
    const payNow = new anchor.BN(2_000_000);

    await program.methods
      .fulfillProofOfPayment(payNow)
      .accounts({
        protocol_signer: admin.publicKey,
        protocol_vault: protocolVaultPda,
        protocol_usdc_account: merchantUsdcAccount, // <--- again
        merchant_usdc_account: merchantUsdcAccount,
        proof_of_payment: proofOfPaymentPda,
        buyer_account: buyerAccountPda,
        merchant_account: merchantAccountPda,
        solend_program: solendProgram,
        solend_reserve: solendReserve,
        reserve_liquidity_supply: reserveLiquiditySupply,
        reserve_collateral_mint: reserveCollateralMint,
        lending_market: lendingMarket,
        lending_market_authority: lendingMarketAuthority,
        protocol_collateral_account: protocolCollateralAccount,
        token_program: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();
  });

  it("Unstake", async () => {
    // unstake => buyer, buyer_account, protocol_vault, protocol_usdc_account, buyer_usdc_account, ...
    const withdrawAmount = new anchor.BN(20_000_000);

    await program.methods
      .unstake(withdrawAmount)
      .accounts({
        buyer: buyer.publicKey,
        buyer_account: buyerAccountPda,
        protocol_vault: protocolVaultPda,
        protocolUsdcAccount: merchantUsdcAccount, // <--- reference the protocol's USDC
        buyerUsdcAccount: buyerUsdcAccount,
        protocolCollateralAccount: protocolCollateralAccount,
        solendProgram: solendProgram,
        solendReserve: solendReserve,
        reserveLiquiditySupply: reserveLiquiditySupply,
        reserveCollateralMint: reserveCollateralMint,
        lendingMarket: lendingMarket,
        lendingMarketAuthority: lendingMarketAuthority,
        token_program: TOKEN_PROGRAM_ID,
      })
      .signers([buyer])
      .rpc();
  });
});
