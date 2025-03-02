use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use solend_sdk::solana_program::program::invoke;
use solend_sdk::{
    self,
    instruction::deposit_reserve_liquidity,  // The official CPI call
};

use crate::state::{BuyerAccount, ProtocolVault};

#[derive(Accounts)]
pub struct StakeAsset<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    // The user's USDC token account (holding actual USDC)
    // TODO: maybe add some constraint here
    #[account(
        mut,
    )]
    pub buyer_usdc_account: Account<'info, TokenAccount>,

    // BuyerAccount storing staked amounts, etc.
    #[account(
        init_if_needed,
        payer = buyer,
        space = BuyerAccount::INIT_SPACE + 8,
        seeds = [b"buyer", buyer.key().as_ref()],
        bump
    )]
    pub buyer_account: Account<'info, BuyerAccount>,

    // ProtocolVault that tracks total staked
    #[account(
        mut,
        seeds = [b"protocol_vault"],
        bump = protocol_vault.bump
    )]
    pub protocol_vault: Account<'info, ProtocolVault>,

    // Solends accounts
    #[account(mut)]
    /// CHECK: This is solend program
    pub solend_program: AccountInfo<'info>,
    /// CHECK: This is the Solend Reserve for USDC. Verified via Solend CPI instructions.
    pub solend_reserve: AccountInfo<'info>,
    /// CHECK: This is the Solend liquidity supply. Verified via Solend CPI instructions.
    pub reserve_liquidity_supply: AccountInfo<'info>,
    /// CHECK: This is the Solend collateral mint for cUSDC. Verified via Solend CPI instructions.
    pub reserve_collateral_mint: AccountInfo<'info>,
    /// CHECK: This is the Solend collateral mint for cUSDC. Verified via Solend CPI instructions.
    pub lending_market: AccountInfo<'info>,
    /// CHECK: This is the Lending Market Authority. Verified via Solend CPI instructions.
    pub lending_market_authority: AccountInfo<'info>,

    // A token account (PDA) to hold the cUSDC on behalf of your program or user
    pub protocol_collateral_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> StakeAsset<'info> {
    pub fn stake(&mut self, amount: u64) -> Result<()> {
        let buyer = &self.buyer;
        let buyer_account = &mut self.buyer_account;
        let protocol_vault = &mut self.protocol_vault;
    
        // Build Solend’s deposit_reserve_liquidity and account_infos instruction
        let deposit_ix = deposit_reserve_liquidity(
            self.solend_program.key(),         // Solend program ID
            amount,                            // How many tokens to deposit
            self.buyer_usdc_account.key(),     // Source USDC token account
            self.protocol_collateral_account.key(), // Where cUSDC/collateral will be minted to
            self.solend_reserve.key(),         // The Solend reserve account
            self.reserve_liquidity_supply.key(),         // Reserve liquidity supply account
            self.reserve_collateral_mint.key(), // Reserve collateral mint
            self.lending_market.key(),         // Lending market account
            self.lending_market_authority.key(),         // Lending market authority
        );

        let account_infos = &[
            self.buyer.to_account_info(),
            self.buyer_usdc_account.to_account_info(),
            self.protocol_collateral_account.to_account_info(),
            self.solend_reserve.to_account_info(),
            self.reserve_liquidity_supply.to_account_info(),
            self.reserve_collateral_mint.to_account_info(),
            self.lending_market.to_account_info(),
            self.lending_market_authority.to_account_info(),
            self.token_program.to_account_info(),
            self.solend_program.to_account_info(),
        ];
    
        // If a PDA needs to sign, you’d do `invoke_signed`. If the user just signs, you do `invoke`.
        invoke(
            &deposit_ix,
            account_infos,
        )?;
    
        buyer_account.buyer = *buyer.key;
        buyer_account.staked_amount += amount;
        buyer_account.unlockable_amount += amount;
    
        protocol_vault.total_staked += amount;
    
        Ok(())
    }
}
