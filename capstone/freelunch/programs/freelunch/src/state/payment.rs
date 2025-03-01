use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ProofOfFuturePayment {
    pub payment_amount: u64, // Amount owed to merchant
    pub locked_collateral: u64,   // How much is locked to generate yield for payment
    pub admin: Pubkey, // Protocol admin managing payouts
    pub buyer: Pubkey, // The buyer responsible for the payment
    pub merchant: Pubkey, // The merchant receiving the payment
    pub completed: u8, // Payment status (0: Pending, 1: Completed)
    pub payment_number: u64, // Payment ID for tracking
    pub amount_fulfilled: u64, // Amount already paid
}
