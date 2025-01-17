import {
  Commitment,
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
} from "@solana/web3.js";
import wallet from "../wba-wallet.json";
import { getOrCreateAssociatedTokenAccount, transfer } from "@solana/spl-token";

// We're going to import our keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

//Create a Solana devnet connection
const commitment: Commitment = "confirmed";
const connection = new Connection("https://api.devnet.solana.com", commitment);

// Mint address
const mint = new PublicKey("E7K69EEoKb9d3SXGqQN2tViX1Nus48zrDT9wNyRDDJqZ");

// Recipient address
const to = new PublicKey("HPVUnjfkQWet7uMDDAtUNbkGZVSjb54GVMjpJwHYZoHT");

(async () => {
  try {
    // Get the token account of the fromWallet address, and if it does not exist, create it
    const fromAtaAddress = await getOrCreateAssociatedTokenAccount(
      connection,
      keypair,
      mint,
      keypair.publicKey,
      false,
      commitment
    );
    // Get the token account of the toWallet address, and if it does not exist, create it
    const toAtaAddress = await getOrCreateAssociatedTokenAccount(
      connection,
      keypair,
      mint,
      to,
      false,
      commitment
    );

    // Transfer the new token to the "toTokenAccount" we just created
    const tx = await transfer(
      connection,
      keypair,
      fromAtaAddress.address,
      toAtaAddress.address,
      keypair.publicKey,
      10000 //0.01 DAV
    );
    console.log(`Your txid: ${tx}`);
  } catch (e) {
    console.error(`Oops, something went wrong: ${e}`);
  }
})();
