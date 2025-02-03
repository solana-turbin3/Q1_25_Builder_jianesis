use anchor_lang::prelude::*;
use anchor_lang::prelude::Pubkey;

#[account]
pub struct Listing {
  pub maker: Pubkey, // the account that made the listing
  pub mint: Pubkey, // the NFT mint account
  pub price: u64,
  pub bump: u8,
}

impl Space for Listing {
  const INIT_SPACE: usize = 8+32+32+8+1;
}