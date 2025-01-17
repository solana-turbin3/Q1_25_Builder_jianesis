import bs58 from "bs58";
import prompt from "prompt-sync";

// Initialize prompt-sync
const input = prompt();

// Function to convert Base58 to Wallet format
function base58ToWallet(): void {
  const base58 = input("Enter your Base58 encoded string: ");
  try {
    const wallet = bs58.decode(base58);
    console.log("Decoded wallet byte array:", Array.from(wallet));
  } catch (error) {
    console.error("Invalid Base58 string:", (error as Error).message);
  }
}

// Function to convert Wallet to Base58 format
function walletToBase58(): void {
  const walletInput = input("Enter your wallet byte array (comma-separated): ");
  try {
    const sanitizedInput = walletInput.replace(/[\[\]]/g, "").trim();

    // Split input into numbers
    const wallet = sanitizedInput.split(",").map((num) => {
      const parsed = parseInt(num.trim(), 10);
      if (isNaN(parsed)) {
        throw new Error(`Invalid number in wallet byte array: "${num.trim()}"`);
      }
      return parsed;
    });

    // Encode wallet byte array into Base58
    const base58 = bs58.encode(Buffer.from(wallet));
    console.log("Base58 encoded string:", base58);
  } catch (error) {
    console.error("Error:", (error as Error).message);
  }
}

// Main menu
function main(): void {
  console.log("Select an option:");
  console.log("1. Convert Base58 to Wallet");
  console.log("2. Convert Wallet to Base58");
  const choice = input("Enter your choice (1 or 2): ");

  if (choice === "1") {
    base58ToWallet();
  } else if (choice === "2") {
    walletToBase58();
  } else {
    console.log("Invalid choice. Exiting.");
  }
}

// Run the main function
main();
