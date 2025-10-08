use anchor_lang::prelude::*;
use crate::state::Config;

/// Initialize the global configuration
pub fn init_config(
    ctx: Context<InitConfig>,
    min_tvl: u64,
    allowed_programs: Vec<Pubkey>,
) -> Result<()> {
    let config = &mut ctx.accounts.config;

    config.admin = ctx.accounts.admin.key();
    config.min_tvl = min_tvl;
    config.allowed_programs = allowed_programs;

    msg!("Config initialized with admin: {}, min_tvl: {}", config.admin, min_tvl);

    Ok(())
}

#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(
        init,
        payer = admin,
        space = Config::LEN,
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}
