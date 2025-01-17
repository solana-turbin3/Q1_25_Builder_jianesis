use bs58;
use std::io::{self, BufRead};

fn base58_to_wallet() {
    println!("Input your private key as base58:");
    let stdin = io::stdin();
    let base58 = stdin.lock().lines().next().unwrap().unwrap();
    println!("Your wallet file is:");
    let wallet = bs58::decode(base58).into_vec().unwrap();
    println!("{:?}", wallet);
}

fn wallet_to_base58() {
    println!("Input your private key as a wallet file byte array:");
    let stdin = io::stdin();
    let wallet = stdin
        .lock()
        .lines()
        .next()
        .unwrap()
        .unwrap()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|s| s.trim().parse::<u8>().unwrap())
        .collect::<Vec<u8>>();
    println!("Your private key is:");
    let base58 = bs58::encode(wallet).into_string();
    println!("{:?}", base58);
}

fn main() {
    println!("Solana Key Format Converter");
    println!("1. Convert Base58 to Wallet format");
    println!("2. Convert Wallet format to Base58");
    println!("Choose an option (1 or 2):");

    let stdin = io::stdin();
    let choice = stdin.lock().lines().next().unwrap().unwrap();

    match choice.as_str() {
        "1" => base58_to_wallet(),
        "2" => wallet_to_base58(),
        _ => println!("Invalid choice!"),
    }
}