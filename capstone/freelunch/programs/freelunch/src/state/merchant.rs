use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct MerchantAccount {
    pub merchant: Pubkey, // The merchant's public key
    pub status: u8, // KYB verification status (0: Pending, 1: Approved)
    pub payment_number: u64, // Number of payments received
    pub amount_transacted: u64, // Total amount of USDC received from buyers
    pub seed:u128,


}