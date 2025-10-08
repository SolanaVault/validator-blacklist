use anchor_lang::prelude::*;
use crate::state::Config;
use crate::error::ValidatorBlacklistError;

/// Update the config settings (min_tvl and allowed_programs)
pub fn update_config(
    ctx: Context<UpdateConfig>,
    min_tvl: Option<u64>,
    allowed_programs: Option<Vec<Pubkey>>,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    
    if let Some(min_tvl) = min_tvl {
        config.min_tvl = min_tvl;
        msg!("Config min_tvl updated to: {}", min_tvl);
    }
    
    if let Some(allowed_programs) = allowed_programs {
        config.allowed_programs = allowed_programs;
        msg!("Config allowed_programs updated");
    }
    
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(
        mut,
        has_one = admin @ ValidatorBlacklistError::UnauthorizedAdmin
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub admin: Signer<'info>,
}
