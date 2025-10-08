use anchor_lang::prelude::*;
use crate::state::Config;
use crate::error::ValidatorBlacklistError;

/// Update the admin of the config
pub fn update_config_admin(
    ctx: Context<UpdateConfigAdmin>,
    new_admin: Pubkey,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    
    config.admin = new_admin;

    msg!("Config admin updated to: {}", new_admin);
    
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateConfigAdmin<'info> {
    #[account(
        mut,
        has_one = admin @ ValidatorBlacklistError::UnauthorizedAdmin
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub admin: Signer<'info>,
}
