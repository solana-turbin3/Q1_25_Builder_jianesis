use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct BuyerAccount {
    pub buyer: Pubkey, // The buyer's public key
    pub staked_amount: u64, // Total amount staked
    pub unlockable_amount: u64, // Amount that can be withdrawn
    pub locked_amount: u64, // Locked amount for pending payments
    pub reward_amount: u64, // Rewards earned from staking
}
