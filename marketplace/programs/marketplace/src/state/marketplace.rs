
use anchor_lang::prelude::*;
use anchor_lang::prelude::Pubkey;

#[account]
pub struct Marketplace {
  pub admin: Pubkey,
  pub fee: u16,
  pub bump: u8,
  pub  treasury_bump:u8,
  pub reward_bump: u8,
  pub name: String
}

impl Space for Marketplace {
  const INIT_SPACE: usize =8+32+2+1+1+1+(4+32);
}