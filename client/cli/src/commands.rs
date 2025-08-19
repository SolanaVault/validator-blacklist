use crate::cli::{Cli, Commands};
use anchor_client::solana_sdk::{
    pubkey::Pubkey, 
    signature::{Keypair, read_keypair_file},
    commitment_config::CommitmentConfig,
    signer::Signer,
};
use anchor_client::solana_account_decoder::UiAccountEncoding;
use anchor_client::{Client, Cluster};
use anchor_lang::AccountDeserialize;
use anyhow::{Result, Context};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{RpcFilterType, Memcmp};
use solana_sdk_ids::system_program;
use validator_blacklist::state::Blacklist;
use std::str::FromStr;
use std::rc::Rc;

pub async fn run_command(cli: Cli) -> Result<()> {
    let program_id = Pubkey::from_str(&cli.program_id)
        .context("Invalid program ID")?;

    match cli.command {
        Commands::List => {
            list_blacklisted_validators(&cli.rpc, &program_id).await?;
        }
        _ => {
            // For commands that require a keypair
            let keypair_path = cli.keypair
                .context("Keypair path is required for this command")?;
            
            let keypair = read_keypair_file(&keypair_path)
                .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?;

            execute_instruction(cli.command, &cli.rpc, &keypair, &program_id).await?;
        }
    }

    Ok(())
}

async fn list_blacklisted_validators(rpc_url: &str, program_id: &Pubkey) -> Result<()> {
    let rpc_client = RpcClient::new(rpc_url.to_string());

    // Get all blacklist accounts
    let accounts = rpc_client.get_program_accounts_with_config(
        program_id,
        RpcProgramAccountsConfig {
            filters: Some(vec![
                RpcFilterType::Memcmp(Memcmp::new_raw_bytes(0, b"blacklist".to_vec())),
            ]),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                commitment: Some(CommitmentConfig::confirmed()),
                ..Default::default()
            },
            ..Default::default()
        },
    )?;

    if accounts.is_empty() {
        println!("No blacklisted validators found.");
        return Ok(());
    }

    println!("Blacklisted Validators:");
    println!("{:<44} {:<10} {:<10}", "Validator Address", "Add Votes", "Remove Votes");
    println!("{}", "-".repeat(70));

    for (pubkey, account) in accounts {
        // Try to deserialize using the borsh trait method directly
        if account.data.len() < 8 {
            eprintln!("Account {} has invalid data length", pubkey);
            continue;
        }

        let mut data = &account.data[8..];
        
        let blacklist = Blacklist::try_deserialize(&mut data).unwrap();
            
        println!(
            "{:<44} {:<10} {:<10}",
            blacklist.validator_identity_address,
            blacklist.tally_add,
            blacklist.tally_remove
        );
    }

    Ok(())
}

async fn execute_instruction(command: Commands, rpc_url: &str, keypair: &Keypair, program_id: &Pubkey) -> Result<()> {
    // Create the Anchor client
    let cluster = Cluster::Custom(rpc_url.to_string(), "ws://localhost:8900".to_string());
    let client = Client::new_with_options(cluster, Rc::new(keypair.insecure_clone()), CommitmentConfig::confirmed());
    let program = client.program(*program_id)?;

    match command {
        Commands::VoteAdd { validator_address, stake_pool, reason, delegation } => {
            println!("Executing VoteAdd for validator: {}", validator_address);
            
            // Parse string arguments to Pubkey
            let validator_pubkey = Pubkey::from_str(&validator_address)
                .context("Invalid validator address")?;
            let stake_pool_pubkey = Pubkey::from_str(&stake_pool)
                .context("Invalid stake pool address")?;
            let delegation_pubkey = if let Some(del) = delegation {
                Some(Pubkey::from_str(&del).context("Invalid delegation address")?)
            } else {
                None
            };
            
            // Calculate PDAs
            let (blacklist_pda, _) = Pubkey::find_program_address(
                &[b"blacklist", validator_pubkey.as_ref()],
                program_id,
            );
            
            let authority = delegation_pubkey.unwrap_or(keypair.pubkey());
            let (vote_add_pda, _) = Pubkey::find_program_address(
                &[b"vote_add", authority.as_ref(), validator_pubkey.as_ref()],
                program_id,
            );

            let mut request = program
                .request()
                .accounts(validator_blacklist::accounts::VoteAdd {
                    blacklist: blacklist_pda,
                    vote_add: vote_add_pda,
                    stake_pool: stake_pool_pubkey,
                    authority,
                    delegation: None,
                    system_program: system_program::id(),
                })
                .args(validator_blacklist::instruction::VoteAdd {
                    validator_identity_address: validator_pubkey,
                    reason,
                });

            // Add delegation account if provided
            if let Some(_delegation_addr) = delegation_pubkey {
                let (delegation_pda, _) = Pubkey::find_program_address(
                    &[b"delegation", stake_pool_pubkey.as_ref(), authority.as_ref()],
                    program_id,
                );
                request = request.accounts(validator_blacklist::accounts::VoteAdd {
                    blacklist: blacklist_pda,
                    vote_add: vote_add_pda,
                    stake_pool: stake_pool_pubkey,
                    authority,
                    delegation: Some(delegation_pda),
                    system_program: system_program::id(),
                });
            }

            let signature = request.send()?;
            println!("Vote to add transaction sent: {}", signature);
        },
        Commands::VoteRemove { validator_address, stake_pool, reason, delegation } => {
            println!("Executing VoteRemove for validator: {}", validator_address);
            
            // Parse string arguments to Pubkey
            let validator_pubkey = Pubkey::from_str(&validator_address)
                .context("Invalid validator address")?;
            let stake_pool_pubkey = Pubkey::from_str(&stake_pool)
                .context("Invalid stake pool address")?;
            let delegation_pubkey = if let Some(del) = delegation {
                Some(Pubkey::from_str(&del).context("Invalid delegation address")?)
            } else {
                None
            };
            
            let (blacklist_pda, _) = Pubkey::find_program_address(
                &[b"blacklist", validator_pubkey.as_ref()],
                program_id,
            );
            
            let authority = delegation_pubkey.unwrap_or(keypair.pubkey());
            let (vote_remove_pda, _) = Pubkey::find_program_address(
                &[b"vote_remove", authority.as_ref(), validator_pubkey.as_ref()],
                program_id,
            );

            let mut request = program
                .request()
                .accounts(validator_blacklist::accounts::VoteRemove {
                    blacklist: blacklist_pda,
                    vote_remove: vote_remove_pda,
                    stake_pool: stake_pool_pubkey,
                    authority,
                    delegation: None,
                    system_program: system_program::id(),
                })
                .args(validator_blacklist::instruction::VoteRemove {
                    validator_identity_address: validator_pubkey,
                    reason,
                });

            // Add delegation account if provided
            if let Some(_delegation_addr) = delegation_pubkey {
                let (delegation_pda, _) = Pubkey::find_program_address(
                    &[b"delegation", stake_pool_pubkey.as_ref(), authority.as_ref()],
                    program_id,
                );
                request = request.accounts(validator_blacklist::accounts::VoteRemove {
                    blacklist: blacklist_pda,
                    vote_remove: vote_remove_pda,
                    stake_pool: stake_pool_pubkey,
                    authority,
                    delegation: Some(delegation_pda),
                    system_program: system_program::id(),
                });
            }

            let signature = request.send()?;
            println!("Vote to remove transaction sent: {}", signature);
        },
        Commands::UnvoteAdd { validator_address, stake_pool, delegation } => {
            println!("Executing UnvoteAdd for validator: {}", validator_address);
            
            // Parse string arguments to Pubkey
            let validator_pubkey = Pubkey::from_str(&validator_address)
                .context("Invalid validator address")?;
            let stake_pool_pubkey = Pubkey::from_str(&stake_pool)
                .context("Invalid stake pool address")?;
            let delegation_pubkey = if let Some(del) = delegation {
                Some(Pubkey::from_str(&del).context("Invalid delegation address")?)
            } else {
                None
            };
            
            let (blacklist_pda, _) = Pubkey::find_program_address(
                &[b"blacklist", validator_pubkey.as_ref()],
                program_id,
            );
            
            let authority = delegation_pubkey.unwrap_or(keypair.pubkey());
            let (vote_add_pda, _) = Pubkey::find_program_address(
                &[b"vote_add", authority.as_ref(), validator_pubkey.as_ref()],
                program_id,
            );

            let mut request = program
                .request()
                .accounts(validator_blacklist::accounts::UnvoteAdd {
                    blacklist: blacklist_pda,
                    vote_add: vote_add_pda,
                    stake_pool: stake_pool_pubkey,
                    authority,
                    delegation: None,
                })
                .args(validator_blacklist::instruction::UnvoteAdd {
                    validator_identity_address: validator_pubkey,
                });

            // Add delegation account if provided
            if let Some(_delegation_addr) = delegation_pubkey {
                let (delegation_pda, _) = Pubkey::find_program_address(
                    &[b"delegation", stake_pool_pubkey.as_ref(), authority.as_ref()],
                    program_id,
                );
                request = request.accounts(validator_blacklist::accounts::UnvoteAdd {
                    blacklist: blacklist_pda,
                    vote_add: vote_add_pda,
                    stake_pool: stake_pool_pubkey,
                    authority,
                    delegation: Some(delegation_pda),
                });
            }

            let signature = request.send()?;
            println!("Unvote add transaction sent: {}", signature);
        },
        Commands::UnvoteRemove { validator_address, stake_pool, delegation } => {
            println!("Executing UnvoteRemove for validator: {}", validator_address);
            
            // Parse string arguments to Pubkey
            let validator_pubkey = Pubkey::from_str(&validator_address)
                .context("Invalid validator address")?;
            let stake_pool_pubkey = Pubkey::from_str(&stake_pool)
                .context("Invalid stake pool address")?;
            let delegation_pubkey = if let Some(del) = delegation {
                Some(Pubkey::from_str(&del).context("Invalid delegation address")?)
            } else {
                None
            };
            
            let (blacklist_pda, _) = Pubkey::find_program_address(
                &[b"blacklist", validator_pubkey.as_ref()],
                program_id,
            );
            
            let authority = delegation_pubkey.unwrap_or(keypair.pubkey());
            let (vote_remove_pda, _) = Pubkey::find_program_address(
                &[b"vote_remove", authority.as_ref(), validator_pubkey.as_ref()],
                program_id,
            );

            let mut request = program
                .request()
                .accounts(validator_blacklist::accounts::UnvoteRemove {
                    blacklist: blacklist_pda,
                    vote_remove: vote_remove_pda,
                    stake_pool: stake_pool_pubkey,
                    authority,
                    delegation: None,
                })
                .args(validator_blacklist::instruction::UnvoteRemove {
                    validator_identity_address: validator_pubkey,
                });

            // Add delegation account if provided
            if let Some(_delegation_addr) = delegation_pubkey {
                let (delegation_pda, _) = Pubkey::find_program_address(
                    &[b"delegation", stake_pool_pubkey.as_ref(), authority.as_ref()],
                    program_id,
                );
                request = request.accounts(validator_blacklist::accounts::UnvoteRemove {
                    blacklist: blacklist_pda,
                    vote_remove: vote_remove_pda,
                    stake_pool: stake_pool_pubkey,
                    authority,
                    delegation: Some(delegation_pda),
                });
            }

            let signature = request.send()?;
            println!("Unvote remove transaction sent: {}", signature);
        },
        Commands::Delegate { stake_pool, delegate } => {
            println!("Executing Delegate for stake pool: {}", stake_pool);
            
            // Parse string arguments to Pubkey
            let stake_pool_pubkey = Pubkey::from_str(&stake_pool)
                .context("Invalid stake pool address")?;
            let delegate_pubkey = Pubkey::from_str(&delegate)
                .context("Invalid delegate address")?;
            
            let (delegation_pda, _) = Pubkey::find_program_address(
                &[b"delegation", stake_pool_pubkey.as_ref(), keypair.pubkey().as_ref()],
                program_id,
            );

            let signature = program
                .request()
                .accounts(validator_blacklist::accounts::Delegate {
                    delegation: delegation_pda,
                    stake_pool: stake_pool_pubkey,
                    manager: keypair.pubkey(),
                    delegate: delegate_pubkey,
                    system_program: system_program::id(),
                })
                .args(validator_blacklist::instruction::Delegate {})
                .send()?;

            println!("Delegate transaction sent: {}", signature);
        },
        Commands::Undelegate { stake_pool } => {
            println!("Executing Undelegate for stake pool: {}", stake_pool);
            
            // Parse string arguments to Pubkey
            let stake_pool_pubkey = Pubkey::from_str(&stake_pool)
                .context("Invalid stake pool address")?;
            
            let (delegation_pda, _) = Pubkey::find_program_address(
                &[b"delegation", stake_pool_pubkey.as_ref(), keypair.pubkey().as_ref()],
                program_id,
            );

            let signature = program
                .request()
                .accounts(validator_blacklist::accounts::Undelegate {
                    delegation: delegation_pda,
                    stake_pool: stake_pool_pubkey,
                    manager: keypair.pubkey(),
                })
                .args(validator_blacklist::instruction::Undelegate {})
                .send()?;

            println!("Undelegate transaction sent: {}", signature);
        },
        Commands::List => {
            unreachable!()
        }
    }

    Ok(())
}
