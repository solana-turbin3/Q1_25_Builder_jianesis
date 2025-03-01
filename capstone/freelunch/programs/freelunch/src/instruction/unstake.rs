use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;

use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"buyer", buyer.key().as_ref()],
        bump
    )]
    pub buyer_account: Account<'info, BuyerAccount>,

    #[account(mut, seeds = [b"protocol_vault"], bump)]
    pub protocol_vault: Account<'info, ProtocolVault>,

    #[account(mut)]
    pub buyer_usdc_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub protocol_usdc_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    // pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn unstake(&mut self, amount: u64) -> Result<()> {
        let buyer_account = &mut self.buyer_account;
        let protocol_vault = &mut self.protocol_vault;
        let buyer_usdc_account = &self.buyer_usdc_account;
        let protocol_usdc_account = &self.protocol_usdc_account;
        let token_program = &self.token_program;

        require!(buyer_account.unlockable_amount >= amount, ErrorCode::InsufficientFunds);

        // Transfer USDC back from protocol vault to buyer
        let cpi_ctx = CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: protocol_usdc_account.to_account_info(),
                to: buyer_usdc_account.to_account_info(),
            },
        );
        transfer(cpi_ctx, amount)?;

        // Update buyer and protocol states
        buyer_account.staked_amount -= amount;
        buyer_account.unlockable_amount -= amount;
        protocol_vault.total_staked -= amount;

        Ok(())
    }
}