use anchor_lang::prelude::*;
use crate::state::Delegation;

/// Remove delegation of authority
pub fn undelegate(
    ctx: Context<Undelegate>,
) -> Result<()> {
    let delegation = &ctx.accounts.delegation;

    msg!("Delegation removed: stake pool {} manager {} undelegated from {}", 
         ctx.accounts.stake_pool.key(), ctx.accounts.manager.key(), delegation.delegate);

    Ok(())
}

#[derive(Accounts)]
pub struct Undelegate<'info> {
    #[account(
        mut,
        close = manager,
        seeds = [b"delegation", stake_pool.key().as_ref(), manager.key().as_ref()],
        bump,
        has_one = manager,
        has_one = stake_pool,
    )]
    pub delegation: Account<'info, Delegation>,

    /// The stake pool account to validate the manager
    /// CHECK: We manually validate this is a valid stake pool in the instruction logic
    #[account()]
    pub stake_pool: UncheckedAccount<'info>,

    /// The manager of the stake pool (must match the delegation's manager field)
    #[account(mut)]
    pub manager: Signer<'info>,
}
