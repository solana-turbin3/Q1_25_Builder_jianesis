use anchor_lang::prelude::*;

declare_id!("8dKSPSMBeoPSsZXcRdsJHVYTJ7B8FVXpwfVWLENqsJRZ");

pub mod contexts;
pub mod state;
pub mod error;
pub mod constant;

pub use constant::*;
pub use contexts::*;
pub use state::*;

#[program]
pub mod nft_staking {
    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>, points_per_stake: u8, max_stake: u8, freeze_period: u32) -> Result<()> {
        ctx.accounts.init_config(points_per_stake, max_stake, freeze_period, &ctx.bumps)
    }

    pub fn initialize_user(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.register_user(&ctx.bumps)
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        ctx.accounts.stake(&ctx.bumps)
    }
}
