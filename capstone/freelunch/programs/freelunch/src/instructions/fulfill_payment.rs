use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, transfer};
use solend_sdk::instruction::redeem_reserve_collateral;
use solend_sdk::solana_program::program::invoke_signed;
use crate::state::*;
use crate::error::ErrorCode;

// We'll assume the protocol or an admin calls this on a schedule (like daily or weekly).
#[derive(Accounts)]
pub struct FulfillProofOfPayment<'info> {
    /// CHECK: This is the protocol's trusted signer; signature verified in instruction.
    #[account(signer)]
    pub protocol_signer: AccountInfo<'info>,

    // The protocol vault (PDA)
    #[account(
        mut,
        seeds = [b"protocol_vault"],
        bump = protocol_vault.bump
    )]
    pub protocol_vault: Account<'info, ProtocolVault>,

    // The protocol's token account that holds USDC (harvested from Solend).
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

    #[account(
        mut,
        seeds = [b"merchant", proof_of_payment.merchant.key().as_ref()],
        bump
    )]
    pub merchant_account: Account<'info, MerchantAccount>,

    // Solends accounts
    #[account(mut)]
    /// CHECK: This is solend program
    pub solend_program: AccountInfo<'info>,
    /// CHECK: This is solend program
    pub solend_reserve: AccountInfo<'info>,
    /// CHECK: This is the Solend liquidity supply. Verified via Solend CPI instructions.
    pub reserve_liquidity_supply: AccountInfo<'info>,
    /// CHECK: This is the Solend collateral mint for cUSDC. Verified via Solend CPI instructions.
    pub reserve_collateral_mint: AccountInfo<'info>,
    /// CHECK: This is the Solend lending market. Verified via Solend CPI instructions.
    pub lending_market: AccountInfo<'info>,
    /// CHECK: This is the Solend lending market authority. Verified via Solend CPI instructions.
    pub lending_market_authority: AccountInfo<'info>,

    pub protocol_collateral_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> FulfillProofOfPayment<'info> {
    pub fn complete_payment(
        &mut self,
        amount_to_pay_now: u64
    ) -> Result<()> {
        let vault_bump = self.protocol_vault.bump;

        // 1) Build a redeem_reserve_collateral CPI instruction
        let redeem_ix = redeem_reserve_collateral(
            self.solend_program.key(),
            amount_to_pay_now, // cUSDC
            self.protocol_collateral_account.key(),
            self.protocol_usdc_account.key(),
            self.solend_reserve.key(),
            self.reserve_liquidity_supply.key(),
            self.reserve_collateral_mint.key(),
            self.lending_market.key(),
            self.lending_market_authority.key(),
        );

        // 2) Gather the accounts required by the redeem instruction
        let redeem_infos = &[
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

        // should be invoke signed
        invoke_signed(
            &redeem_ix,
            redeem_infos,
            &[&[b"protocol_vault", &[vault_bump]]],
        )?;

    // 2) Transfer from protocol_usdc_account to merchant_usdc_account
        //    using the vault's authority (PDA).
        let cpi_progmram = self.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.protocol_usdc_account.to_account_info(),
            to: self.merchant_usdc_account.to_account_info(),
            authority: self.protocol_vault.to_account_info(),
        };

        let binding = [vault_bump];
        let vault_seeds = &[&[b"protocol_vault".as_ref(), &binding][..]];

        let cpi_ctx = CpiContext::new_with_signer(
            cpi_progmram,
            cpi_accounts,
            vault_seeds,
            
        );

        transfer(cpi_ctx.with_signer(vault_seeds), amount_to_pay_now)?;

        // 3) Update PoF, buyer, merchant as usual
        let proof = &mut self.proof_of_payment;
        require!(proof.completed == 0, ErrorCode::PaymentAlreadyCompleted);

        let remain = proof.payment_amount
            .checked_sub(proof.amount_fulfilled)
            .ok_or(ErrorCode::Unauthorized)?;
        let pay_now = std::cmp::min(amount_to_pay_now, remain);

        proof.amount_fulfilled = proof.amount_fulfilled
            .checked_add(pay_now)
            .ok_or(ErrorCode::Unauthorized)?;

        // If fully paid, free the buyer's locked collateral
        if proof.amount_fulfilled >= proof.payment_amount {
            proof.completed = 1;
            self.buyer_account.locked_amount = self.buyer_account.locked_amount
                .checked_sub(proof.locked_collateral)
                .ok_or(ErrorCode::Unauthorized)?;
            self.buyer_account.unlockable_amount = self.buyer_account.unlockable_amount
                .checked_add(proof.locked_collateral)
                .ok_or(ErrorCode::Unauthorized)?;
        }

        self.merchant_account.amount_transacted = self.merchant_account
            .amount_transacted
            .checked_add(pay_now)
            .ok_or(ErrorCode::Unauthorized)?;
        Ok(())
        }
}
