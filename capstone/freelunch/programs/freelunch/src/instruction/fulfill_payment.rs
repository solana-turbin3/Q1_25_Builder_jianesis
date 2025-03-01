use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, transfer};
use solend_sdk::instruction::redeem_reserve_collateral;
use solend_sdk::solana_program::program::invoke_signed;
use crate::state::*;
use crate::error::ErrorCode;

// We'll assume the protocol or an admin calls this on a schedule (like daily or weekly).
#[derive(Accounts)]
pub struct FulfillProofOfPayment<'info> {
    // Some authority who can fulfill payments—could be a PDA or an admin
    // that orchestrates these partial payments
    #[account(mut)]
    pub payer: Signer<'info>,

    // The protocol's token account that holds USDC (harvested from Solend).
    // We'll transfer from here to the merchant.
    #[account(mut)]
    pub protocol_usdc_account: Account<'info, TokenAccount>,

    // The merchant's USDC token account (the final destination of the funds).
    #[account(mut)]
    pub merchant_usdc_account: Account<'info, TokenAccount>,

    // The ProofOfFuturePayment record to fulfill
    #[account(mut)]
    pub proof_of_payment: Account<'info, ProofOfFuturePayment>,

    // The buyer's account, so we can unlock collateral if we fully pay the PoF
    #[account(mut)]
    pub buyer_account: Account<'info, BuyerAccount>,

    // The protocol vault might be needed if we have constraints on who can sign for the transfer
    // or if the vault is a PDA that must sign. For simplicity, we skip that here.
    #[account(
        mut,
        seeds = [b"protocol_vault"],
        bump
    )]
    pub protocol_vault: Account<'info, ProtocolVault>,

    // SolendAccount
    pub solend_program: AccountInfo<'info>,
    #[account(mut)]
    pub protocol_collateral_account: Account<'info, TokenAccount>,

    // Solends accounts
    #[account(mut)]
    pub solend_reserve: AccountInfo<'info>,
    pub reserve_liquidity_supply: AccountInfo<'info>,
    pub reserve_collateral_mint: AccountInfo<'info>,
    pub lending_market: AccountInfo<'info>,
    pub lending_market_authority: AccountInfo<'info>,

    // Standard programs
    pub token_program: Program<'info, Token>,

    #[account(
        mut,
        seeds = [b"merchant", proof_of_payment.merchant.key().as_ref()],
        bump
    )]
    pub merchant_account: Account<'info, MerchantAccount>,
}

impl<'info> FulfillProofOfPayment<'info> {
    pub fn complete_payment(
        &mut self,
        amount_to_pay_now: u64
    ) -> Result<()> {

        // 1) Build a redeem_reserve_collateral CPI instruction
        let redeem_ix = redeem_reserve_collateral(
            self.solend_program.key(),
            amount_to_pay_now,  // amount to redeem
            self.protocol_collateral_account.key(),
            self.protocol_usdc_account.key(),
            self.solend_reserve.key(),
            self.reserve_liquidity_supply.key(),
            self.reserve_collateral_mint.key(),
            self.lending_market.key(),
            self.lending_market_authority.key(),
        );

        // 2) Gather the accounts required by the redeem instruction
        let account_infos = &[
            self.payer.to_account_info(),
            self.protocol_collateral_account.to_account_info(),
            self.protocol_usdc_account.to_account_info(),
            self.solend_reserve.to_account_info(),
            // add required Lending Market + Market Authority + token_program + ...
        ];

        // should be invoke signed
        invoke_signed(
            &redeem_ix,
            account_infos,
            &[&[b"protocol_vault", &[self.protocol_vault.bump]]],
        )?;


        let proof = &mut self.proof_of_payment;
        let buyer_account = &mut self.buyer_account;
        
        // 1. Check if PoF is already completed
        require!(proof.completed == 0, ErrorCode::PaymentAlreadyCompleted);

        // 2. Bound `amount_to_pay_now` by what remains
        let remaining_due = proof.payment_amount
            .checked_sub(proof.amount_fulfilled)
            .ok_or(ErrorCode::Unauthorized)?; // shouldn't happen if completed=0
        let pay_now = std::cmp::min(amount_to_pay_now, remaining_due);

        // 3. Transfer from protocol_usdc_account to merchant_usdc_account
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.protocol_usdc_account.to_account_info(),
                to: self.merchant_usdc_account.to_account_info(),
                authority: self.payer.to_account_info(),
            },
        );
        transfer(cpi_ctx, pay_now)?;

        // 4. Update PoF fields
        proof.amount_fulfilled = proof.amount_fulfilled
            .checked_add(pay_now)
            .ok_or(ErrorCode::Unauthorized)?;

        // 5. If fully paid, mark completed & unlock collateral
        if proof.amount_fulfilled >= proof.payment_amount {
            proof.completed = 1;

            // Buyer can now unlock that portion of collateral
            // Subtract locked_collateral from buyer's locked_amount
            // Add it back to buyer's unlockable_amount
            // Because the PoF is satisfied, the collateral can be freed

            buyer_account.locked_amount = buyer_account.locked_amount
                .checked_sub(proof.locked_collateral)
                .ok_or(ErrorCode::Unauthorized)?;
            buyer_account.unlockable_amount = buyer_account.unlockable_amount
                .checked_add(proof.locked_collateral)
                .ok_or(ErrorCode::Unauthorized)?;
        }
        // 6. Update merchant_account
        self.merchant_account.amount_transacted = self.merchant_account
            .amount_transacted
            .checked_add(pay_now)
            .ok_or(ErrorCode::Unauthorized)?;    

        Ok(())
    }
}
