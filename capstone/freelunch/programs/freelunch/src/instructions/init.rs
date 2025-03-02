use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = ProtocolVault::INIT_SPACE + 8,
        seeds = [b"protocol_vault"],
        bump
    )]
    pub protocol_vault: Account<'info, ProtocolVault>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn init(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.protocol_vault.set_inner(
            ProtocolVault {
                admin: *self.admin.key,
                total_staked: 0,
                total_rewards: 0,
                pending_payments: 0,
                bump: bumps.protocol_vault,
            }
        );
        Ok(())
    }
}
