use anchor_lang::prelude::*;
use crate::authority_checks;
use crate::stake_pool_helpers::{deserialize_stake_pool_with_checks, validate_stake_pool_config};
use crate::state::{Blacklist, VoteRemoveFromBlacklist, Delegation, Config};
use crate::error::ValidatorBlacklistError;

/// Remove a previously cast vote to remove a validator from the blacklist
pub fn unvote_remove(
    ctx: Context<UnvoteRemove>,
    validator_identity_address: Pubkey,
) -> Result<()> {

    let blacklist = &mut ctx.accounts.blacklist;

    let stake_pool = deserialize_stake_pool_with_checks(&ctx.accounts.stake_pool.try_borrow_data()?)?;

    // Validate stake pool meets config requirements
    validate_stake_pool_config(
        &stake_pool,
        &ctx.accounts.stake_pool.owner,
        &ctx.accounts.config,
    )?;

    // Validate the authority
    authority_checks::check_authority(
        ctx.accounts.delegation.as_deref(), 
        &ctx.accounts.stake_pool.key(), 
        &stake_pool,
        &ctx.accounts.authority.key())?;

    // Decrease the tally
    blacklist.tally_remove = blacklist.tally_remove.checked_sub(1)
        .ok_or(ValidatorBlacklistError::MathUnderflow)?;

    msg!("Removed vote to remove validator {} cast by stake pool {}", 
         validator_identity_address, ctx.accounts.stake_pool.key());

    Ok(())
}

#[derive(Accounts)]
#[instruction(validator_identity_address: Pubkey)]
pub struct UnvoteRemove<'info> {
    /// Global configuration account
    #[account()]
    pub config: Account<'info, Config>,

    /// The stake pool account to validate the authority
    /// CHECK: We manually validate this is a valid stake pool in the instruction logic
    pub stake_pool: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"blacklist", config.key().as_ref(), validator_identity_address.as_ref()],
        bump
    )]
    pub blacklist: Account<'info, Blacklist>,

    #[account(
        mut,
        close = authority,
        seeds = [b"vote_remove", config.key().as_ref(), stake_pool.key().as_ref(), validator_identity_address.as_ref()],
        bump
    )]
    pub vote_remove: Account<'info, VoteRemoveFromBlacklist>,

    /// Optional delegation account - if present, authority must be the delegate
    #[account(
        seeds = [b"delegation", config.key().as_ref(), stake_pool.key().as_ref()],
        bump
    )]
    pub delegation: Option<Account<'info, Delegation>>,

    /// The authority (either manager or delegated authority)
    #[account(mut)]
    pub authority: Signer<'info>,
}
