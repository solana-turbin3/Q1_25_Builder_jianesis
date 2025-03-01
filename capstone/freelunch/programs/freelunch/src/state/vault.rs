use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ProtocolVault {
    pub admin: Pubkey, // Admin of the protocol
    pub total_staked: u64, // Total USDC staked across all users
    pub total_rewards: u64, // Total rewards generated from staking
    pub pending_payments: u64, // Total outstanding Proof of Future Payments
    pub bump: u8,
}
