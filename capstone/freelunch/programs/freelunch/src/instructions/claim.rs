use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, transfer};
use crate::state::*;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct MerchantClaim<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,

    #[account(
        mut,
        constraint = proof_of_payment.merchant == merchant.key() @ ErrorCode::Unauthorized,
    )]
    pub proof_of_payment: Account<'info, ProofOfFuturePayment>,

    #[account(mut)]
    pub buyer_account: Account<'info, BuyerAccount>,

    #[account(mut)]
    pub protocol_usdc_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub merchant_usdc_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"merchant", proof_of_payment.merchant.key().as_ref()],
        bump
    )]
    pub merchant_account: Account<'info, MerchantAccount>,

    #[account(
        mut,
        seeds = [b"protocol_vault"],
        bump = protocol_vault.bump
    )]
    pub protocol_vault: Account<'info, ProtocolVault>,

    pub token_program: Program<'info, Token>,
}


impl<'info> MerchantClaim<'info> {
    /// The merchant can claim up to `amount_to_claim` from the PoF.
    /// If the PoF can be partially paid, they get partial. If it covers the entire remainder, the PoF is closed.
    pub fn merchant_claim(&mut self, amount_to_claim: u64) -> Result<()> {
        let proof = &mut self.proof_of_payment;
        let buyer_account = &mut self.buyer_account;
        let protocol_vault: &Account<'info, ProtocolVault> = &self.protocol_vault;

        // Check if already completed
        require!(proof.completed == 0, ErrorCode::PaymentAlreadyCompleted);

        // Check remaining
        let remaining_due = proof.payment_amount
            .checked_sub(proof.amount_fulfilled)
            .ok_or(ErrorCode::Unauthorized)?; // shouldn't happen if completed=0
        let claim_now = std::cmp::min(amount_to_claim, remaining_due);

        // Transfer from the protocol’s USDC account to the merchant’s USDC account
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.protocol_usdc_account.to_account_info(),
            to: self.merchant_usdc_account.to_account_info(),
            authority: self.protocol_vault.to_account_info(), 
        };

        let signer_seeds: &[&[u8]] = &[&b"protocol_vault"[..], &[protocol_vault.bump]];

        let binding =&[signer_seeds];
        let cpi_ctx = CpiContext::new_with_signer(
            cpi_program,
            cpi_accounts,
            binding
        );
        transfer(cpi_ctx, claim_now)?;

        proof.amount_fulfilled = proof.amount_fulfilled
            .checked_add(claim_now)
            .ok_or(ErrorCode::Unauthorized)?;

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
