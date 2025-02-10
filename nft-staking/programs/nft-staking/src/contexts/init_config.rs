use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::state::StakeConfig;

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
  #[account(mut)]
  pub admin: Signer<'info>,

  #[account(
    init, 
    payer = admin,
    space=StakeConfig::INIT_SPACE+8,
    seeds = [b"config", admin.key().as_ref()],
    bump,
  )]
  pub config: Account<'info, StakeConfig>,

  #[account(
    init,
    payer = admin,
    seeds = [b"rewards", config.key().as_ref()],
    bump,
    mint::decimals = 6,
    mint::authority = config,
    mint::freeze_authority = config,
  )]
  pub rewards_mint: Account<'info, Mint>,
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, Token>,
}

impl <'info> InitializeConfig<'info> {
  pub fn init_config(&mut self, points_per_stake: u8, max_stake: u8, freeze_period: u32, bumps: &InitializeConfigBumps) -> Result<()> {
    self.config.set_inner(StakeConfig {
      bump: bumps.config,
      rewards_bump: bumps.rewards_mint,
      max_stake,
      freeze_period,
      points_per_stake,
    });
    Ok(())
  }
}