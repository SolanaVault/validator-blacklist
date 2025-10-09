#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod error;
pub mod stake_pool_helpers;
pub mod authority_checks;

use instructions::*;

declare_id!("Fu4zvEKjgxWjaQifp7fyghKJfk6HzUCaJRvoGffJBm6Q");

#[program]
pub mod validator_blacklist {
    use super::*;

    /// Initialize the global configuration
    pub fn init_config(
        ctx: Context<InitConfig>,
        min_tvl: u64,
        allowed_programs: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::init_config::init_config(ctx, min_tvl, allowed_programs)
    }

    /// Update the admin of the config
    pub fn update_config_admin(
        ctx: Context<UpdateConfigAdmin>,
        new_admin: Pubkey,
    ) -> Result<()> {
        instructions::update_config_admin::update_config_admin(ctx, new_admin)
    }

    /// Update the config settings
    pub fn update_config(
        ctx: Context<UpdateConfig>,
        min_tvl: Option<u64>,
        allowed_programs: Option<Vec<Pubkey>>,
    ) -> Result<()> {
        instructions::update_config::update_config(ctx, min_tvl, allowed_programs)
    }

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
