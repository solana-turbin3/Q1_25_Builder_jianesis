use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, transfer};
use solend_sdk::instruction::redeem_reserve_collateral;
use solend_sdk::solana_program::program::invoke_signed;

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

    #[account(
        mut,
        seeds = [b"protocol_vault"],
        bump = protocol_vault.bump
    )]
    pub protocol_vault: Account<'info, ProtocolVault>,

    /// The protocol’s USDC account (where redeemed USDC goes).
    #[account(mut)]
    pub protocol_usdc_account: Account<'info, TokenAccount>,

    /// The buyer’s USDC account to receive the withdrawn USDC.
    #[account(mut)]
    pub buyer_usdc_account: Account<'info, TokenAccount>,

    /// cUSDC token account (owned by the protocol vault) that holds staked collateral
    #[account(mut)]
    pub protocol_collateral_account: Account<'info, TokenAccount>,

    // Solend + related accounts
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

    pub token_program: Program<'info, Token>,
}

impl<'info> Withdraw<'info> {
    pub fn unstake(&mut self, amount: u64) -> Result<()> {
        let buyer_account = &mut self.buyer_account;
        let protocol_vault: &Account<'info, ProtocolVault> = &self.protocol_vault;

        // 1) Check buyer has enough unlockable
        require!(buyer_account.unlockable_amount >= amount, ErrorCode::InsufficientFunds);

        // 2) Redeem from Solend cUSDC to USDC
        let redeem_ix = redeem_reserve_collateral(
            self.solend_program.key(),
            amount, // This is cUSDC amount to redeem; we assume 1:1
            self.protocol_collateral_account.key(),
            self.protocol_usdc_account.key(),
            self.solend_reserve.key(),
            self.reserve_liquidity_supply.key(),
            self.reserve_collateral_mint.key(),
            self.lending_market.key(),
            self.lending_market_authority.key(),
        );

        let account_infos = &[
            self.protocol_collateral_account.to_account_info(),
            self.protocol_usdc_account.to_account_info(),
            self.solend_reserve.to_account_info(),
            self.reserve_liquidity_supply.to_account_info(),
            self.reserve_collateral_mint.to_account_info(),
            self.lending_market.to_account_info(),
            self.lending_market_authority.to_account_info(),
            self.token_program.to_account_info(),
            self.solend_program.to_account_info(),
        ];

        // 3) Because the protocol vault (PDA) should be the authority of `protocol_collateral_account`,
        //    we do invoke_signed with the seeds for the vault.
        invoke_signed(
            &redeem_ix,
            account_infos,
            &[&[b"protocol_vault", &[protocol_vault.bump]]],
        )?;

        // 4) USDC in protocol_usdc_account, do a normal SPL transfer to buyer
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.protocol_usdc_account.to_account_info(),
            to: self.buyer_usdc_account.to_account_info(),
            authority: self.protocol_vault.to_account_info(),
        };
        let vault_seeds: &[&[u8]] = &[&b"protocol_vault"[..], &[protocol_vault.bump]];
        let binding = [vault_seeds];
        let cpi_ctx = CpiContext::new_with_signer(
            cpi_program,
            cpi_accounts,
            &binding
        );
        transfer(cpi_ctx, amount)?;


        // 5) Update local BNPL state
        buyer_account.staked_amount = buyer_account.staked_amount.checked_sub(amount)
            .ok_or(ErrorCode::InsufficientFunds)?;
        buyer_account.unlockable_amount = buyer_account.unlockable_amount.checked_sub(amount)
            .ok_or(ErrorCode::InsufficientFunds)?;

        // Decrement total_staked in the protocol vault
         protocol_vault.total_staked.checked_sub(amount)
        .ok_or(ErrorCode::InsufficientFunds)?;

        Ok(())
    }
}
