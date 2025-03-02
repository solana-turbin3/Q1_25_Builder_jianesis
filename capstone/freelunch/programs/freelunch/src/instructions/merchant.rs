use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct MerchantInit<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,

    #[account(
        init,
        payer = merchant,
        space = MerchantAccount::INIT_SPACE + 8,
        seeds = [b"merchant", merchant.key().as_ref()],
        bump
    )]
    pub merchant_account: Account<'info, MerchantAccount>,

    pub system_program: Program<'info, System>,
}

impl<'info> MerchantInit<'info> {
    pub fn merchant_init(&mut self, seed: u128) -> Result<()> {
      self.merchant_account.set_inner(
        MerchantAccount {
            merchant: *self.merchant.key,
            status: 1,
            payment_number: 0,
            amount_transacted: 0,
            seed,
        }
      );
      
      Ok(())
    }
}
