use anchor_lang::prelude::*;
use solend_sdk::math::{Decimal, TryAdd, TryDiv, TryMul, TrySub, WAD};
use solend_sdk::solana_program::program_pack::Pack;
use solend_sdk::state::Reserve;
use crate::state::*;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct CreateProofOfPayment<'info> {
    // The admin verifies the purchase off-chain and signs to create PoF
    #[account(mut)]
    pub admin: Signer<'info>,

    // The buyer's staking account
    #[account(
        mut,
        constraint = buyer_account.staked_amount > 0 @ ErrorCode::InsufficientStake
    )]
    pub buyer_account: Account<'info, BuyerAccount>,

    // unique seed (buyer + merchant + merchant.payment_number)
    #[account(
        init,
        payer = admin,
        space = ProofOfFuturePayment::INIT_SPACE + 8,
        seeds = [b"proof_of_payment", buyer_account.buyer.key().as_ref(), merchant_account.merchant.key().as_ref(), merchant_account.payment_number.to_le_bytes().as_ref()],
        bump
    )]
    pub proof_of_payment: Account<'info, ProofOfFuturePayment>,

    // Merchant account to track how many payments have been assigned
    #[account(
        mut,
        seeds = [b"merchant", merchant_account.merchant.key().as_ref()],
        bump
    )]
    pub merchant_account: Account<'info, MerchantAccount>,

    // The merchant's account (just for verification)
    #[account()]
    pub merchant: SystemAccount<'info>,

    /// The Solend reserve account holding interest rate data
    #[account(mut)]
    pub solend_reserve: AccountInfo<'info>,


    pub system_program: Program<'info, System>,
}

impl<'info> CreateProofOfPayment<'info> {
    // create proof of payment
    pub fn purchase(
        &mut self,
        purchase_amount: u64,   // e.g. 5 USDC
        buffer_bps: u64         // e.g. 500 for an extra 5% buffer
    ) -> Result<()> {
        let solend_reserve_data = self.solend_reserve.data.borrow();
        let reserve: Reserve = Reserve::unpack(&solend_reserve_data)
            .map_err(|_| error!(ErrorCode::Unauthorized))?;

        // Derive deposit APY from reserve fields
        let deposit_apy_bps = Self::compute_deposit_apy_bps(&reserve)?;


        let buyer_account = &mut self.buyer_account;
        let merchant_account = &mut self.merchant_account;
        let proof = &mut self.proof_of_payment;

        require!(merchant_account.status == 1, ErrorCode::InvalidMerchant);
        require!(purchase_amount > 0, ErrorCode::InvalidPurchaseAmount);

        // 1. Calculate base locked collateral based on APY
        // locked_value = purchase_amount * 10000 / deposit_apy_bps
        require!(deposit_apy_bps > 0, ErrorCode::InvalidAPY);

        let base_locked_value = purchase_amount
            .checked_mul(10000)
            .ok_or(ErrorCode::Unauthorized)?
            .checked_div(deposit_apy_bps) // APY derived from reserve
            .ok_or(ErrorCode::Unauthorized)?;

        // 2. Add buffer
        // locked_value = base_locked_value * (10000 + buffer_bps) / 10000
        let locked_value_with_buffer = base_locked_value
            .checked_mul(10000 + buffer_bps)
            .ok_or(ErrorCode::Unauthorized)?
            .checked_div(10000)
            .ok_or(ErrorCode::Unauthorized)?;

        // Ensure the buyer has enough unlockable funds
        require!(buyer_account.unlockable_amount >= locked_value_with_buffer, ErrorCode::InsufficientFunds);

        // 3. Lock that collateral
        buyer_account.unlockable_amount = buyer_account.unlockable_amount
            .checked_sub(locked_value_with_buffer)
            .ok_or(ErrorCode::InsufficientFunds)?;
        buyer_account.locked_amount = buyer_account.locked_amount
            .checked_add(locked_value_with_buffer)
            .ok_or(ErrorCode::Unauthorized)?;

        // 4. Fill out the proof of payment
        proof.payment_amount = purchase_amount; // e.g. 5 USDC
        proof.locked_collateral = locked_value_with_buffer;  // e.g. 52 or 53 USDC w/ buffer
        proof.admin = *self.admin.key;
        proof.buyer = buyer_account.buyer;
        proof.merchant = merchant_account.merchant;
        proof.completed = 0; // 0 => not paid, 1 => completed
        proof.payment_number = merchant_account.payment_number;
        proof.amount_fulfilled = 0;

        // 5. Increment the merchant's payment_number
        merchant_account.payment_number += 1;
        Ok(())
    }

    /// Use the fields from `reserve` to derive deposit APY. 
    /// Often deposit_apy ~ utilization * borrow_rate * (1 - protocol_cut).
    fn compute_deposit_apy_bps(reserve: &Reserve) -> Result<u64> {
        // 1) Compute utilization = borrowed_amount / (borrowed_amount + available_amount)
        // 2) Compute borrow_rate from config fields, e.g. `reserve.current_borrow_rate()`
        // 3) deposit_apy = borrow_rate * utilization * (1 - protocol_take_rate)
        // Convert to basis points.
        
        let total_supply = reserve.liquidity.borrowed_amount_wads // as Decimal
            .try_add(Decimal::from(reserve.liquidity.available_amount))
            .map_err(|_| ErrorCode::Unauthorized)?;

        if total_supply == Decimal::zero() {
            // no liquidity => APY is zero or negligible
            return Ok(0);
        }

        let utilization = reserve.liquidity.borrowed_amount_wads
            .try_div(total_supply)
            .map_err(|_| ErrorCode::Unauthorized)?;

        // This is simplified. The real logic is in `reserve.current_borrow_rate()`
        // But let's pretend we do something like:
        let borrow_rate_pct = reserve.config.optimal_borrow_rate; // e.g. 10 => 10%
        let borrow_rate = Decimal::from(borrow_rate_pct as u64).try_div(Decimal::from(100u64))
            .map_err(|_| ErrorCode::Unauthorized)?;
        
        let protocol_take_rate = Decimal::from(reserve.config.protocol_take_rate as u64)
            .try_div(Decimal::from(100u64))
            .map_err(|_| ErrorCode::Unauthorized)?;

        // deposit_apy (decimal) = utilization * borrow_rate * (1 - protocol_take_rate)
        let deposit_apy_decimal = utilization
            .try_mul(borrow_rate)
            .map_err(|_| ErrorCode::Unauthorized)?
            .try_mul(Decimal::one().try_sub(protocol_take_rate)?)
            .map_err(|_| ErrorCode::Unauthorized)?;

        // Convert decimal to basis points (1.0 -> 10000 bps)
        let deposit_apy_bps = deposit_apy_decimal
            .to_scaled_val()
            .map_err(|_| ErrorCode::Unauthorized)?
            .checked_div((WAD / 10_000).into())
            .ok_or(ErrorCode::Unauthorized)?;
        
        Ok(deposit_apy_bps.try_into().unwrap())
    }
}
