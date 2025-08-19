use anchor_lang::prelude::*;
use spl_stake_pool::state::StakePool;

use crate::{state::Delegation, error::ValidatorBlacklistError};

pub fn check_authority(delegation: Option<&Delegation>, stake_pool_address: &Pubkey, stake_pool: &StakePool, authority: &Pubkey) -> Result<()> {

    if let Some(delegation) = delegation {

        // Using delegated authority
   
        // Make sure the delegation is valid for the stake pool         
        require_keys_eq!(
            delegation.stake_pool,
            *stake_pool_address,
            ValidatorBlacklistError::InvalidStakePool
        );

        // Make sure the delegate has signed
        require_keys_eq!(
            delegation.delegate,
            *authority,
            ValidatorBlacklistError::InvalidDelegate
        );

        // Make sure the manager matches (this should never fail as it should be checked in the 
        // delegate instruction, but adding for safety)
        require_keys_eq!(
            delegation.manager,
            stake_pool.manager,
            ValidatorBlacklistError::InvalidDelegate
        );

    } else {
        
        // Direct authority, i.e. it should be signed by the stake pool manager
        
        require_keys_eq!(
            stake_pool.manager,
            *authority,
            ValidatorBlacklistError::InvalidManager
        );
    };

    Ok(())
}