use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};

#[derive(Accounts)]
pub struct Initialize<'info>{
  #[account(mut)]
  pub user: Signer<'info>, // house?

  #[account(
      mut,
      seeds = [b"vault", user.key().as_ref()],
      bump
  )]
  pub vault: SystemAccount<'info>,
  pub system_program: Program<'info, System>
}

impl<'info> Initialize<'info>{
  pub fn init(&mut self, amount:u64) -> Result<()>{
    let cpi_accounts = Transfer{
      from: self.user.to_account_info(),
      to: self.vault.to_account_info(),
    };
    let cpi_program = self.system_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    transfer(cpi_ctx, amount)?;
    Ok(())
  }
}