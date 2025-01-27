use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::CloseAccount;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked};
use crate::state::Escrow;



#[derive(Accounts)]
pub struct Refund<'info> {

#[account(mut)]
pub maker: Signer<'info>,
pub token_mint_a: InterfaceAccount<'info, Mint>,
pub token_mint_b: InterfaceAccount<'info, Mint>,    

#[account(
  mut,
  associated_token::mint = token_mint_a,
  associated_token::authority = maker,
)]
pub maker_mint_a_ata: InterfaceAccount<'info, TokenAccount>,

#[account(
  mut,
  close=maker,
  seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],  
  bump= escrow.bump
)]
pub escrow: Account<'info, Escrow>,

#[account(
  mut,
  associated_token::mint=mint_a,
  associated_token::authority = escrow,  
  // associated_token::token_program = token_program,   //not needed because anchor under the hood knows how to get the token program
)]
pub vault: InterfaceAccount<'info, TokenAccount>, //could name it escrow_token_account
pub token_program: Interface<'info, TokenInterface>,
pub associated_token_program: Program<'info, AssociatedToken>,
pub system_program: Program<'info, System>,
}


impl<'info> Refund<'info> {
  pub fn withdraw(&mut self) -> Result<()> {
    let cpi_program = self.token_program.to_account_info();

    let cpi_accounts = TransferChecked{
      from: self.vault.to_account_info(),
      mint: self.token_mint_a.to_account_info(),
      to: self.maker_mint_a_ata.to_account_info(),
      authority: self.escrow.to_account_info(),
    };

    let seed_binding = self.escrow.seed.to_le_bytes();
    let maker_binding = self.escrow.maker.to_bytes();
    let bump_binding = self.escrow.bump;
    let seeds = [b"escrow", &seed_binding, &maker_binding, &[bump_binding]];

    let signer_seeds = &[&seeds];

    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

    transfer_checked(cpi_ctx, self.vault.amount, self.token_mint_a.decimals)?;

    Ok(())
  }

  pub fn close(&mut self) -> Result<()> {
    let cpi_program = self.token_program.to_account_info();
    
    let cpi_accounts = CloseAccount{
       authority: self.escrow.to_account_info(),
       account: self.vault.to_account_info(),
       destination: self.taker.to_account_info()
    };
 
    let seed_binding = self.escrow.seed.to_le_bytes();
    let maker_binding = self.escrow.maker.to_bytes();
    let bump_binding = self.escrow.bump;
    let seeds = [b"escrow", &seed_binding, &maker_binding, &[bump_binding]];
 
    let signer_seeds = &[&seeds];
 
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
 
     close_account(cpi_ctx)?;
     Ok(())
  }
}