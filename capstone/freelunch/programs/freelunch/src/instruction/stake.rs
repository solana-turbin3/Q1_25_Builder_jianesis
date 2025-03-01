use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, transfer};
use solend_sdk::solana_program::program::invoke;
use solend_sdk::{
    self,
    instruction::deposit_reserve_liquidity,  // The official CPI call
    // state::Reserve as SolendReserveState,     // If you want to parse on-chain data
};

use crate::state::{BuyerAccount, ProtocolVault};

use crate::error::ErrorCode; // If you want to handle custom errors

#[derive(Accounts)]
pub struct StakeAsset<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    // The user's USDC token account (holding actual USDC)
    #[account(
        mut,
        constraint = buyer_usdc_account.mint == usdc_mint.key() @ ErrorCode::Unauthorized
    )]
    pub buyer_usdc_account: Account<'info, TokenAccount>,

    // This is the USDC mint - might be verified in your context
    pub usdc_mint: AccountInfo<'info>,

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
        bump
    )]
    pub protocol_vault: Account<'info, ProtocolVault>,

    // The Solend Reserve for USDC. Typically you’d pass in more accounts:
    // e.g. Reserve liquidity, lending market, lending market authority, etc.

    // Solends accounts
    #[account(mut)]
    pub solend_reserve: AccountInfo<'info>,
    pub reserve_liquidity_supply: AccountInfo<'info>,
    pub reserve_collateral_mint: AccountInfo<'info>,
    pub lending_market: AccountInfo<'info>,
    pub lending_market_authority: AccountInfo<'info>,

    // The collateral token mint or cUSDC mint Solend uses (if relevant).
    // To receive cUSDC into a program-owned account, pass that here.
    #[account(mut)]
    pub solend_collateral_mint: AccountInfo<'info>,

    // A token account (PDA) to hold the cUSDC on behalf of your program or user
    // so that the user can't move it unilaterally
    #[account(mut)]
    pub protocol_collateral_account: Account<'info, TokenAccount>,


    // The Solend program itself
    pub solend_program: AccountInfo<'info>,

    // Required for SPL token transfers
    pub token_program: Program<'info, Token>,

    // System program for creating new accounts if needed
    pub system_program: Program<'info, System>,
}

impl<'info> StakeAsset<'info> {
    // pub fn stake1(&mut self, amount: u64) -> Result<()> {
    //     // Access all the accounts more succinctly
    //     let buyer = &self.buyer;
    //     let buyer_account = &mut self.buyer_account;
    //     let protocol_vault = &mut self.protocol_vault;
    //     let user_usdc_account = &self.buyer_usdc_account;
    //     let protocol_collateral_account = &self.protocol_collateral_account;
    //     let token_program = &self.token_program;

    //     // 1. Transfer USDC from user to Solend Reserve (via CPI).
    //     //    In practice, Solend has a specialized deposit instruction. 
    //     //    This is a simplified approach showing a direct SPL transfer
    //     //    but you'll want to call Solend's depositReserveLiquidity instruction instead.

    //     // Pseudocode: build & invoke deposit instruction to solend:
    //     // let deposit_ix = solend_instruction::deposit_reserve_liquidity(...);
    //     // let accounts = [...];
    //     // invoke(&deposit_ix, &accounts)?;

    //     // For demonstration, just do an SPL token transfer from user to your program,
    //     // representing that the tokens end up under your program's control.
    //     let cpi_ctx = CpiContext::new(
    //         token_program.to_account_info(),
    //         Transfer {
    //             from: user_usdc_account.to_account_info(),
    //             to: protocol_collateral_account.to_account_info(),
    //             authority: buyer.to_account_info(),
    //         },
    //     );
    //     transfer(cpi_ctx, amount)?;

    //     // 2. Update your program's internal state
    //     buyer_account.buyer = *buyer.key;
    //     buyer_account.staked_amount += amount;
    //     buyer_account.unlockable_amount += amount;
    //     protocol_vault.total_staked += amount;

    //     Ok(())
    // }

    pub fn stake(&mut self, amount: u64) -> Result<()> {
        let buyer = &self.buyer;
        let buyer_account = &mut self.buyer_account;
        let protocol_vault = &mut self.protocol_vault;
    
        // 1) Build Solend’s deposit_reserve_liquidity instruction
        // Make sure you pass in exactly the accounts Solend expects
        // in the correct order. Check Solend docs for the latest param list.
    
        let deposit_ix = deposit_reserve_liquidity(
            self.solend_program.key(),         // Solend program ID
            amount,                            // How many tokens to deposit
            self.buyer_usdc_account.key(),     // Source USDC token account
            self.protocol_collateral_account.key(), // Where cUSDC/collateral will be minted to
            self.solend_reserve.key(),         // The Solend reserve account
            self.reserve_liquidity_supply.key(),         // Reserve liquidity supply account
            self.solend_collateral_mint.key(), // Reserve collateral mint
            self.lending_market.key(),         // Lending market account
            self.lending_market_authority.key(),         // Lending market authority
        );
    
        // 2) Gather the AccountInfos into a slice 
        //    (all the accounts that deposit_reserve_liquidity expects)
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
    
        // 3) If a PDA needs to sign, you’d do `invoke_signed`. If the user just signs, you do `invoke`.
        //    If the buyer is the authority, you might do:
        invoke(
            &deposit_ix,
            account_infos,
        )?;
    
        // 4) Now that the deposit is successful, you can update your local state.
        buyer_account.buyer = *buyer.key;
        buyer_account.staked_amount += amount;
        buyer_account.unlockable_amount += amount;
    
        protocol_vault.total_staked += amount;
    
        Ok(())
    }
}
