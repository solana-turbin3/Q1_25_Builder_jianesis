use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, transfer};
use crate::state::*;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct MerchantClaim<'info> {
    /// The merchant claiming their funds.
    #[account(mut)]
    pub merchant: Signer<'info>,

    /// The PoF must belong to this merchant.
    #[account(
        mut,
        constraint = proof_of_payment.merchant == merchant.key() @ ErrorCode::Unauthorized,
    )]
    pub proof_of_payment: Account<'info, ProofOfFuturePayment>,

    /// The buyer account associated with this PoF, so we can unlock collateral if fully paid.
    #[account(mut)]
    pub buyer_account: Account<'info, BuyerAccount>,

    /// Protocol’s token account (holding USDC). We’ll transfer from here to the merchant.
    #[account(mut)]
    pub protocol_usdc_account: Account<'info, TokenAccount>,

    /// The merchant’s USDC token account (where funds go).
    #[account(mut)]
    pub merchant_usdc_account: Account<'info, TokenAccount>,

    /// The protocol vault, in case we need to check or sign with a PDA. Omitted if you have no constraints.
    #[account(mut)]
    pub protocol_vault: Account<'info, ProtocolVault>,

    /// Standard programs.
    pub token_program: Program<'info, Token>,
}


impl<'info> MerchantClaim<'info> {
    /// The merchant can claim up to `amount_to_claim` from the PoF.
    /// If the PoF can be partially paid, they get partial. If it covers the entire remainder, the PoF is closed.
    pub fn merchant_claim(&mut self, amount_to_claim: u64) -> Result<()> {
        let proof = &mut self.proof_of_payment;
        let buyer_account = &mut self.buyer_account;

        // 1. Check if already completed
        require!(proof.completed == 0, ErrorCode::PaymentAlreadyCompleted);

        // 2. Figure out how much remains
        let remaining_due = proof.payment_amount
            .checked_sub(proof.amount_fulfilled)
            .ok_or(ErrorCode::Unauthorized)?; // shouldn't happen if completed=0
        let claim_now = std::cmp::min(amount_to_claim, remaining_due);

        // 3. Transfer from the protocol’s USDC account to the merchant’s USDC account
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.protocol_usdc_account.to_account_info(),
                to: self.merchant_usdc_account.to_account_info(),
                authority: self.merchant.to_account_info(), 
                // If your protocol uses a PDA authority, replace 
                // 'ctx.accounts.merchant.to_account_info()' with that
                // and require a CPI signature or another approach.
            },
        );
        transfer(cpi_ctx, claim_now)?;

        // 4. Update PoF fields
        proof.amount_fulfilled = proof.amount_fulfilled
            .checked_add(claim_now)
            .ok_or(ErrorCode::Unauthorized)?;

        // 5. If fully paid, mark completed and unlock the collateral for the buyer
        if proof.amount_fulfilled >= proof.payment_amount {
            proof.completed = 1;
            buyer_account.locked_amount = buyer_account.locked_amount
                .checked_sub(proof.locked_collateral)
                .ok_or(ErrorCode::Unauthorized)?;
            buyer_account.unlockable_amount = buyer_account.unlockable_amount
                .checked_add(proof.locked_collateral)
                .ok_or(ErrorCode::Unauthorized)?;
        }

        Ok(())
    }
}