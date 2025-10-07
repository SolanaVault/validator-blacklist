#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod error;
pub mod stake_pool_helpers;
pub mod authority_checks;

use instructions::*;

declare_id!("C7662BVQCwLuorurd8vXohNczQuMHMhDqZ4JcMMge77d");

#[program]
pub mod validator_blacklist {
    use super::*;

    /// Delegate authority from a stake pool manager to another address
    pub fn delegate(
        ctx: Context<Delegate>,
    ) -> Result<()> {
        instructions::delegate::delegate(ctx)
    }

    /// Remove delegation of authority
    pub fn undelegate(
        ctx: Context<Undelegate>,
    ) -> Result<()> {
        instructions::undelegate::undelegate(ctx)
    }

    /// Vote to add a validator to the blacklist
    pub fn vote_add(
        ctx: Context<VoteAdd>,
        validator_identity_address: Pubkey,
        reason: String,
    ) -> Result<()> {
        instructions::vote_add::vote_add(ctx, validator_identity_address, reason)
    }

    /// Vote to remove a validator from the blacklist
    pub fn vote_remove(
        ctx: Context<VoteRemove>,
        validator_identity_address: Pubkey,
        reason: String,
    ) -> Result<()> {
        instructions::vote_remove::vote_remove(ctx, validator_identity_address, reason)
    }

    /// Remove a previously cast vote to add a validator to the blacklist
    pub fn unvote_add(
        ctx: Context<UnvoteAdd>,
        validator_identity_address: Pubkey,
    ) -> Result<()> {
        instructions::unvote_add::unvote_add(ctx, validator_identity_address)
    }

    /// Remove a previously cast vote to remove a validator from the blacklist
    pub fn unvote_remove(
        ctx: Context<UnvoteRemove>,
        validator_identity_address: Pubkey,
    ) -> Result<()> {
        instructions::unvote_remove::unvote_remove(ctx, validator_identity_address)
    }
}
