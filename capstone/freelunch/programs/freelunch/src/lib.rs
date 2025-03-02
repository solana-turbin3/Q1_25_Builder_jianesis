pub mod instructions;
pub mod state;
pub mod error;

pub use instructions::*;

use anchor_lang::prelude::*;


declare_id!("AXnYea6Je9Ui31N6cY8y2oPfppETrh5sr6U31B5A77VQ");

#[program]
pub mod freelunch {
    use super::*;

    /// 1) Initialize the protocol vault
    pub fn init(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.init(&ctx.bumps)
    }

    /// 2) Stake into protocol
    pub fn stake(ctx: Context<StakeAsset>, amount: u64) -> Result<()> {
        ctx.accounts.stake(amount)
    }

    /// 3) Unstake from protocol
    pub fn unstake(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.unstake(amount)
    }

    /// 4) Merchant Initialize
    pub fn merchant_init(ctx: Context<MerchantInit>, seed: u128) -> Result<()> {
        ctx.accounts.merchant_init(seed)
    }

    /// 5) Create a proof-of-payment (purchase)
    pub fn create_proof_of_payment(
        ctx: Context<CreateProofOfPayment>,
        purchase_amount: u64,
        buffer_bps: u64
    ) -> Result<()> {
        ctx.accounts.purchase(purchase_amount, buffer_bps)
    }

    /// 6) Fulfill an outstanding proof-of-payment with yield (admin or crank usage)
    pub fn fulfill_proof_of_payment(
        ctx: Context<FulfillProofOfPayment>,
        amount_to_pay_now: u64
    ) -> Result<()> {
        ctx.accounts.complete_payment(amount_to_pay_now)
    }

    /// 7) Merchant claims partial or full payment
    pub fn merchant_claim(
        ctx: Context<MerchantClaim>,
        amount_to_claim: u64
    ) -> Result<()> {
        ctx.accounts.merchant_claim(amount_to_claim)
    }

}
