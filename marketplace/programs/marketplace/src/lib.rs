pub mod error;
pub mod contexts;
pub mod state;

use anchor_lang::prelude::*;

pub use contexts::*;
pub use state::*;


declare_id!("EW9gy4mgEdAhERJ4TZPmeszE99N6wRd8QdDszatGMMs4");

#[program]
pub mod marketplace {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String, fee: u16) -> Result<()> {
        ctx.accounts.init(name, fee, &ctx.bumps)
    }

    pub fn list(ctx: Context<List>, price: u64) -> Result<()> {
        ctx.accounts.create_listing(price, &ctx.bumps)?;
        ctx.accounts.deposit_nft()
    }

    pub fn purchase(ctx: Context<Purchase>) -> Result<()> {
        ctx.accounts.pay()?;
        ctx.accounts.transfer_nft()?;
        ctx.accounts.close_mint_vault()
    }

    pub fn delist(ctx: Context<Delist>) -> Result<()> {
        ctx.accounts.withdraw_nft()?;
        ctx.accounts.close_mint_vault()
    }
}