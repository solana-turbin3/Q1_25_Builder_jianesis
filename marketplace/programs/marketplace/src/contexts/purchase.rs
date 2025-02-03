use anchor_lang::{
  prelude::*,
  system_program::{transfer, Transfer},
};
use anchor_spl::{
  associated_token::AssociatedToken,
  token_interface::{
      close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
      TransferChecked,
  },
};

// a taker come similar to escrow, they gonna try to buy the NFT.

use crate::state::{Listing, Marketplace};


#[derive(Accounts)]
pub struct Purchase<'info>{
  #[account(mut)]
  pub taker: Signer<'info>,

  #[account(mut)]
  pub maker: SystemAccount<'info>,

  #[account(
    seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
    bump = marketplace.bump
  )]
  pub marketplace:Account<'info,Marketplace>,
  pub maker_nft_mint: InterfaceAccount<'info, Mint>,

  #[account(
    init_if_needed,
    payer=taker, 
    associated_token::mint = maker_nft_mint, 
    associated_token::authority = taker
  )]
  pub taker_ata: InterfaceAccount<'info, TokenAccount>,

  #[account(
    mut,
    associated_token::mint= maker_nft_mint,
    associated_token::authority=maker
  )]
  pub vault: InterfaceAccount<'info, TokenAccount>,

  #[account(
    mut,
    seeds = [marketplace.key().as_ref(), maker_nft_mint.key().as_ref()],
    bump = listing.bump  
  )]
  pub listing: Account<'info,Listing>,

  #[account(
    mut,
    seeds=[b"treasury", marketplace.key().as_ref()],
    bump = marketplace.treasury_bump,
  )]
  pub treasury: SystemAccount<'info>,

  #[account(
    mut,
    seeds=[b"treasury", marketplace.key().as_ref()],
    bump = marketplace.reward_bump,
    mint::authority=marketplace,
    mint::decimals=6,
  )]
  pub rewards_mint: InterfaceAccount<'info, Mint>,

  pub system_program: Program<'info, System>,
  pub token_program: Interface<'info, TokenInterface>,
  pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Purchase<'info> {
  // send sol
  // transfer the nft
  // close the accounts
  pub fn pay(&mut self)->Result<()>{
    let cpi_program = self.system_program.to_account_info();
    let cpi_accounts = Transfer {
      from: self.taker.to_account_info(),
      to: self.maker.to_account_info(),
    };

    let cpi_ctx=CpiContext::new(cpi_program, cpi_accounts);

    let fee = self.marketplace.fee as u64;
    let amount = self.listing.price - fee; // maker pay the fee since maker receiving less
  
    transfer(cpi_ctx, amount)?;

    let cpi_program = self.system_program.to_account_info();

    let cpi_accounts = Transfer {
      from: self.taker.to_account_info(),
      to: self.treasury.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    transfer(cpi_ctx, fee)?;

    Ok(())
  }

  pub fn transfer_nft(&mut self)->Result<()>{ 
    let cpi_program = self.token_program.to_account_info();
    let cpi_accounts = TransferChecked {
      from: self.vault.to_account_info(),
      mint: self.maker_nft_mint.to_account_info(),
      to: self.taker_ata.to_account_info(),
      authority: self.listing.to_account_info(),
    };


    let seeds = &[
      &self.marketplace.key().to_bytes()[..], 
      &self.maker_nft_mint.key().to_bytes()[..],
      &[self.listing.bump]
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

    transfer_checked(cpi_ctx,1, 0)?;
    Ok(())
  }

  pub fn close_mint_vault(&mut self)->Result<()>{
    let seeds = &[
      &self.marketplace.key().to_bytes()[..], 
      &self.maker_nft_mint.key().to_bytes()[..],
      &[self.listing.bump]
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_program = self.token_program.to_account_info();

    let accounts = CloseAccount {
      account: self.vault.to_account_info(),
      destination: self.maker.to_account_info(),
      authority: self.listing.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(cpi_program, accounts, signer_seeds);

    close_account(cpi_ctx)?;
    Ok(())
  }
}