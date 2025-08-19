use anchor_lang::prelude::*;
use spl_stake_pool::state::StakePool;

use crate::stake_pool_helpers::deserialize_stake_pool_with_checks;
use crate::state::Delegation;
use crate::error::ValidatorBlacklistError;

/// Delegate authority from a stake pool manager to another address
pub fn delegate(
    ctx: Context<Delegate>,
) -> Result<()> {

    // Deserialize the SPL Stake Pool
    let stake_pool_state: StakePool = deserialize_stake_pool_with_checks(
        &ctx.accounts.stake_pool.try_borrow_data()?)?;
    
    // Validate the stake pool manager that was passed in to us
    require_keys_eq!(
        stake_pool_state.manager,
        ctx.accounts.manager.key(),
        ValidatorBlacklistError::InvalidManager
    );

    let delegation = &mut ctx.accounts.delegation;
    let clock = Clock::get()?;

    // Initialize delegation
    delegation.stake_pool = ctx.accounts.stake_pool.key();
    delegation.manager = ctx.accounts.manager.key();
    delegation.delegate = ctx.accounts.delegate.key();
    delegation.timestamp = clock.unix_timestamp;

    msg!("Delegation created: stake pool {} manager {} delegated to {}", 
         ctx.accounts.stake_pool.key(), ctx.accounts.manager.key(), ctx.accounts.delegate.key());

    Ok(())
}

#[derive(Accounts)]
pub struct Delegate<'info> {
    /// The stake pool account to validate the manager
    /// CHECK: We manually validate this is a valid stake pool in the instruction logic
    #[account()]
    pub stake_pool: UncheckedAccount<'info>,

    #[account(
        init,
        payer = manager,
        space = Delegation::LEN,
        seeds = [b"delegation", stake_pool.key().as_ref(), manager.key().as_ref()],
        bump
    )]
    pub delegation: Account<'info, Delegation>,

    /// The manager of the stake pool (must match the stake pool's manager field)
    #[account(mut)]
    pub manager: Signer<'info>,

    /// The address to delegate authority to
    /// CHECK: This is just the target of delegation, no validation needed
    pub delegate: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
