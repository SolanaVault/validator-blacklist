use crate::cli::{Cli, Commands};
use anchor_client::solana_sdk::{
    pubkey::Pubkey, 
    signature::{Keypair, read_keypair_file},
    commitment_config::CommitmentConfig,
    signer::Signer,
};
use solana_sdk::transaction::Transaction;
use anchor_client::solana_account_decoder::UiAccountEncoding;
use anchor_client::{Client, Cluster};
use anchor_lang::{AccountDeserialize, Discriminator};
use anyhow::{Result, Context};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{RpcFilterType, Memcmp};
use solana_sdk_ids::system_program;
use validator_blacklist::state::Blacklist;
use std::str::FromStr;
use std::rc::Rc;
use anchor_lang::__private::base64;

pub fn run_command(cli: Cli) -> Result<()> {
    let program_id = Pubkey::from_str(&cli.program_id)
        .context("Invalid program ID")?;

    match cli.command {
        Commands::List => {
            list_blacklisted_validators(&cli.rpc, &program_id)?;
        }
        Commands::Delegate { config, stake_pool, delegate, output, manager } => {
            handle_delegate_command(&cli.rpc, &program_id, config, stake_pool, delegate, output, manager, cli.keypair)?;
        }
        Commands::Undelegate { config, stake_pool, output, manager } => {
            handle_undelegate_command(&cli.rpc, &program_id, config, stake_pool, output, manager, cli.keypair)?;
        }
        Commands::CreateConfig { config, min_tvl, allowed_programs, output, manager } => {
            handle_create_config_command(&cli.rpc, &program_id, config, min_tvl, allowed_programs, output, manager, cli.keypair)?;
        }
        Commands::UpdateConfig { config, min_tvl, allowed_programs, output, manager } => {
            handle_update_config_command(&cli.rpc, &program_id, config, min_tvl, allowed_programs, output, manager, cli.keypair)?;
        }
        Commands::UpdateConfigAdmin { config, new_admin, output, manager } => {
            handle_update_config_admin_command(&cli.rpc, &program_id, config, new_admin, output, manager, cli.keypair)?;
        }
        Commands::VoteAdd { config, validator_address, stake_pool, reason, delegation, output, manager } => {
            handle_vote_add_command(&cli.rpc, &program_id, config, validator_address, stake_pool, reason, delegation, output, manager, cli.keypair)?;
        }
        Commands::VoteRemove { config, validator_address, stake_pool, reason, delegation, output, manager } => {
            handle_vote_remove_command(&cli.rpc, &program_id, config, validator_address, stake_pool, reason, delegation, output, manager, cli.keypair)?;
        }
        Commands::UnvoteAdd { config, validator_address, stake_pool, delegation, output, manager } => {
            handle_unvote_add_command(&cli.rpc, &program_id, config, validator_address, stake_pool, delegation, output, manager, cli.keypair)?;
        }
        Commands::UnvoteRemove { config, validator_address, stake_pool, delegation, output, manager } => {
            handle_unvote_remove_command(&cli.rpc, &program_id, config, validator_address, stake_pool, delegation, output, manager, cli.keypair)?;
        }
    }

    Ok(())
}

fn list_blacklisted_validators(rpc_url: &str, program_id: &Pubkey) -> Result<()> {
    let rpc_client = RpcClient::new(rpc_url.to_string());

    // Get all blacklist accounts
    let accounts = rpc_client.get_program_accounts_with_config(
        program_id,
        RpcProgramAccountsConfig {
            filters: Some(vec![
                RpcFilterType::Memcmp(Memcmp::new_raw_bytes(0, Blacklist::DISCRIMINATOR.to_vec())),
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

        let mut data = account.data.as_slice();
        
        let blacklist = Blacklist::try_deserialize(&mut data)?;
            
        println!(
            "{:<44} {:<10} {:<10}",
            blacklist.validator_identity_address,
            blacklist.tally_add,
            blacklist.tally_remove
        );
    }

    Ok(())
}

fn handle_delegate_command(rpc_url: &str, program_id: &Pubkey, config: String, stake_pool: String, delegate: String, output: String, manager: Option<String>, keypair_option: Option<String>) -> Result<()> {
    let config_pubkey = Pubkey::from_str(&config).context("Invalid config address")?;
    let stake_pool_pubkey = Pubkey::from_str(&stake_pool).context("Invalid stake pool address")?;
    let delegate_pubkey = Pubkey::from_str(&delegate).context("Invalid delegate address")?;

    match output.as_str() {
        "execute" => {
            let keypair_path = keypair_option.context("Keypair path is required for execute mode")?;
            let keypair = read_keypair_file(&keypair_path)
                .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?;

            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(keypair.insecure_clone()), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let (delegation_pda, _) = Pubkey::find_program_address(
                &[b"delegation", config_pubkey.as_ref(), stake_pool_pubkey.as_ref()],
                program_id,
            );

            let signature = program
                .request()
                .accounts(validator_blacklist::accounts::Delegate {
                    config: config_pubkey,
                    stake_pool: stake_pool_pubkey,
                    delegation: delegation_pda,
                    manager: keypair.pubkey(),
                    delegate: delegate_pubkey,
                    system_program: system_program::id(),
                })
                .args(validator_blacklist::instruction::Delegate {})
                .send()?;

            println!("Delegate transaction sent: {}", signature);
        },
        "base58" => {
            let manager_pubkey = manager.context("Manager pubkey is required when output is base58")?;
            let manager_pubkey = Pubkey::from_str(&manager_pubkey).context("Invalid manager pubkey")?;

            let dummy_keypair = Keypair::new();
            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(dummy_keypair), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let (delegation_pda, _) = Pubkey::find_program_address(
                &[b"delegation", config_pubkey.as_ref(), stake_pool_pubkey.as_ref()],
                program_id,
            );

            let ixs = program
                .request()
                .accounts(validator_blacklist::accounts::Delegate {
                    config: config_pubkey,
                    stake_pool: stake_pool_pubkey,
                    delegation: delegation_pda,
                    manager: manager_pubkey,
                    delegate: delegate_pubkey,
                    system_program: system_program::id(),
                })
                .args(validator_blacklist::instruction::Delegate {})
                .instructions()?;

            let tx = Transaction::new_with_payer(&ixs, Some(&manager_pubkey));
            let serialized = bincode::serialize(&tx).context("Failed to serialize transaction")?;
            let base58_tx = bs58::encode(serialized.clone()).into_string();
            let base64_tx = base64::encode(serialized);

            println!("{}", base58_tx);
            println!("Base64: {}", base64_tx);
        },
        _ => {
            return Err(anyhow::anyhow!("Invalid output format. Use 'execute' or 'base58'"));
        }
    }

    Ok(())
}


fn handle_undelegate_command(rpc_url: &str, program_id: &Pubkey, config: String, stake_pool: String, output: String, manager: Option<String>, keypair_option: Option<String>) -> Result<()> {
    let config_pubkey = Pubkey::from_str(&config).context("Invalid config address")?;
    let stake_pool_pubkey = Pubkey::from_str(&stake_pool).context("Invalid stake pool address")?;

    match output.as_str() {
        "execute" => {
            let keypair_path = keypair_option.context("Keypair path is required for execute mode")?;
            let keypair = read_keypair_file(&keypair_path)
                .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?;

            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(keypair.insecure_clone()), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let (delegation_pda, _) = Pubkey::find_program_address(
                &[b"delegation", config_pubkey.as_ref(), stake_pool_pubkey.as_ref()],
                program_id,
            );

            let signature = program
                .request()
                .accounts(validator_blacklist::accounts::Undelegate {
                    config: config_pubkey,
                    stake_pool: stake_pool_pubkey,
                    delegation: delegation_pda,
                    manager: keypair.pubkey(),
                })
                .args(validator_blacklist::instruction::Undelegate {})
                .send()?;

            println!("Undelegate transaction sent: {}", signature);
        },
        "base58" => {
            let manager_pubkey = manager.context("Manager pubkey is required when output is base58")?;
            let manager_pubkey = Pubkey::from_str(&manager_pubkey).context("Invalid manager pubkey")?;

            let dummy_keypair = Keypair::new();
            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(dummy_keypair), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let (delegation_pda, _) = Pubkey::find_program_address(
                &[b"delegation", config_pubkey.as_ref(), stake_pool_pubkey.as_ref()],
                program_id,
            );

            let ixs = program
                .request()
                .accounts(validator_blacklist::accounts::Undelegate {
                    config: config_pubkey,
                    stake_pool: stake_pool_pubkey,
                    delegation: delegation_pda,
                    manager: manager_pubkey,
                })
                .args(validator_blacklist::instruction::Undelegate {})
                .instructions()?;

            let tx = Transaction::new_with_payer(&ixs, Some(&manager_pubkey));
            let serialized = bincode::serialize(&tx).context("Failed to serialize transaction")?;
            let base58_tx = bs58::encode(serialized).into_string();

            println!("{}", base58_tx);
        },
        _ => {
            return Err(anyhow::anyhow!("Invalid output format. Use 'execute' or 'base58'"));
        }
    }

    Ok(())
}

fn handle_create_config_command(rpc_url: &str, program_id: &Pubkey, config: String, min_tvl: u64, allowed_programs: Vec<String>, output: String, manager: Option<String>, keypair_option: Option<String>) -> Result<()> {
    let allowed_program_pubkeys: Result<Vec<Pubkey>> = allowed_programs
        .iter()
        .map(|p| Pubkey::from_str(p).context(format!("Invalid program address: {}", p)))
        .collect();
    let allowed_program_pubkeys = allowed_program_pubkeys?;

    match output.as_str() {
        "execute" => {
            let keypair_path = keypair_option.context("Keypair path is required for execute mode")?;
            let keypair = read_keypair_file(&keypair_path)
                .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?;

            let config_keypair = read_keypair_file(&config)
                .map_err(|e| anyhow::anyhow!("Failed to read config keypair file: {}", e))?;

            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(keypair.insecure_clone()), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let signature = program
                .request()
                .instruction(solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
                .signer(&config_keypair)
                .accounts(validator_blacklist::accounts::InitConfig {
                    config: config_keypair.pubkey(),
                    admin: keypair.pubkey(),
                    system_program: system_program::id(),
                })
                .args(validator_blacklist::instruction::InitConfig {
                    min_tvl,
                    allowed_programs: allowed_program_pubkeys,
                })
                .send()?;

            println!("CreateConfig transaction sent: {}", signature);
            println!("Config account: {}", config_keypair.pubkey());
        },
        "base58" => {
            let manager_pubkey = manager.context("Manager pubkey is required when output is base58")?;
            let manager_pubkey = Pubkey::from_str(&manager_pubkey).context("Invalid manager pubkey")?;

            let config_keypair = read_keypair_file(&config)
                .map_err(|e| anyhow::anyhow!("Failed to read config keypair file: {}", e))?;

            let dummy_keypair = Keypair::new();
            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(dummy_keypair), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let ixs = program
                .request()
                .instruction(solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
                .accounts(validator_blacklist::accounts::InitConfig {
                    config: config_keypair.pubkey(),
                    admin: manager_pubkey,
                    system_program: system_program::id(),
                })
                .args(validator_blacklist::instruction::InitConfig {
                    min_tvl,
                    allowed_programs: allowed_program_pubkeys,
                })
                .instructions()?;

            let mut tx = Transaction::new_with_payer(&ixs, Some(&manager_pubkey));
            tx.partial_sign(&[&config_keypair], solana_sdk::hash::Hash::default());
            let serialized = bincode::serialize(&tx).context("Failed to serialize transaction")?;
            let base58_tx = bs58::encode(serialized).into_string();

            println!("{}", base58_tx);
        },
        _ => {
            return Err(anyhow::anyhow!("Invalid output format. Use 'execute' or 'base58'"));
        }
    }

    Ok(())
}

fn handle_update_config_command(rpc_url: &str, program_id: &Pubkey, config: String, min_tvl: Option<u64>, allowed_programs: Option<Vec<String>>, output: String, manager: Option<String>, keypair_option: Option<String>) -> Result<()> {
    let config_pubkey = Pubkey::from_str(&config).context("Invalid config address")?;

    let allowed_program_pubkeys = if let Some(programs) = allowed_programs {
        let pubkeys: Result<Vec<Pubkey>> = programs
            .iter()
            .map(|p| Pubkey::from_str(p).context(format!("Invalid program address: {}", p)))
            .collect();
        Some(pubkeys?)
    } else {
        None
    };

    match output.as_str() {
        "execute" => {
            let keypair_path = keypair_option.context("Keypair path is required for execute mode")?;
            let keypair = read_keypair_file(&keypair_path)
                .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?;

            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(keypair.insecure_clone()), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let signature = program
                .request()
                .instruction(solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
                .accounts(validator_blacklist::accounts::UpdateConfig {
                    config: config_pubkey,
                    admin: keypair.pubkey(),
                })
                .args(validator_blacklist::instruction::UpdateConfig {
                    min_tvl,
                    allowed_programs: allowed_program_pubkeys,
                })
                .send()?;

            println!("UpdateConfig transaction sent: {}", signature);
        },
        "base58" => {
            let manager_pubkey = manager.context("Manager pubkey is required when output is base58")?;
            let manager_pubkey = Pubkey::from_str(&manager_pubkey).context("Invalid manager pubkey")?;

            let dummy_keypair = Keypair::new();
            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(dummy_keypair), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let ixs = program
                .request()
                .instruction(solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
                .accounts(validator_blacklist::accounts::UpdateConfig {
                    config: config_pubkey,
                    admin: manager_pubkey,
                })
                .args(validator_blacklist::instruction::UpdateConfig {
                    min_tvl,
                    allowed_programs: allowed_program_pubkeys,
                })
                .instructions()?;

            let tx = Transaction::new_with_payer(&ixs, Some(&manager_pubkey));
            let serialized = bincode::serialize(&tx).context("Failed to serialize transaction")?;
            let base58_tx = bs58::encode(serialized).into_string();

            println!("{}", base58_tx);
        },
        _ => {
            return Err(anyhow::anyhow!("Invalid output format. Use 'execute' or 'base58'"));
        }
    }

    Ok(())
}

fn handle_update_config_admin_command(rpc_url: &str, program_id: &Pubkey, config: String, new_admin: String, output: String, manager: Option<String>, keypair_option: Option<String>) -> Result<()> {
    let config_pubkey = Pubkey::from_str(&config).context("Invalid config address")?;
    let new_admin_pubkey = Pubkey::from_str(&new_admin).context("Invalid new admin address")?;

    match output.as_str() {
        "execute" => {
            let keypair_path = keypair_option.context("Keypair path is required for execute mode")?;
            let keypair = read_keypair_file(&keypair_path)
                .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?;

            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(keypair.insecure_clone()), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let signature = program
                .request()
                .instruction(solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
                .accounts(validator_blacklist::accounts::UpdateConfigAdmin {
                    config: config_pubkey,
                    admin: keypair.pubkey(),
                })
                .args(validator_blacklist::instruction::UpdateConfigAdmin {
                    new_admin: new_admin_pubkey,
                })
                .send()?;

            println!("UpdateConfigAdmin transaction sent: {}", signature);
        },
        "base58" => {
            let manager_pubkey = manager.context("Manager pubkey is required when output is base58")?;
            let manager_pubkey = Pubkey::from_str(&manager_pubkey).context("Invalid manager pubkey")?;

            let dummy_keypair = Keypair::new();
            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(dummy_keypair), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let ixs = program
                .request()
                .instruction(solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
                .accounts(validator_blacklist::accounts::UpdateConfigAdmin {
                    config: config_pubkey,
                    admin: manager_pubkey,
                })
                .args(validator_blacklist::instruction::UpdateConfigAdmin {
                    new_admin: new_admin_pubkey,
                })
                .instructions()?;

            let tx = Transaction::new_with_payer(&ixs, Some(&manager_pubkey));
            let serialized = bincode::serialize(&tx).context("Failed to serialize transaction")?;
            let base58_tx = bs58::encode(serialized).into_string();

            println!("{}", base58_tx);
        },
        _ => {
            return Err(anyhow::anyhow!("Invalid output format. Use 'execute' or 'base58'"));
        }
    }

    Ok(())
}

fn handle_vote_add_command(rpc_url: &str, program_id: &Pubkey, config: String, validator_address: String, stake_pool: String, reason: String, delegation: Option<String>, output: String, manager: Option<String>, keypair_option: Option<String>) -> Result<()> {
    let config_pubkey = Pubkey::from_str(&config).context("Invalid config address")?;
    let validator_pubkey = Pubkey::from_str(&validator_address).context("Invalid validator address")?;
    let stake_pool_pubkey = Pubkey::from_str(&stake_pool).context("Invalid stake pool address")?;
    let delegation_pubkey = if let Some(del) = delegation {
        Some(Pubkey::from_str(&del).context("Invalid delegation address")?)
    } else {
        None
    };

    let (blacklist_pda, _) = Pubkey::find_program_address(
        &[b"blacklist", config_pubkey.as_ref(), validator_pubkey.as_ref()],
        program_id,
    );

    let (vote_add_pda, _) = Pubkey::find_program_address(
        &[b"vote_add", config_pubkey.as_ref(), stake_pool_pubkey.as_ref(), validator_pubkey.as_ref()],
        program_id,
    );

    let delegation_pda = delegation_pubkey.map(|_| {
        Pubkey::find_program_address(
            &[b"delegation", config_pubkey.as_ref(), stake_pool_pubkey.as_ref()],
            program_id,
        ).0
    });

    match output.as_str() {
        "execute" => {
            let keypair_path = keypair_option.context("Keypair path is required for execute mode")?;
            let keypair = read_keypair_file(&keypair_path)
                .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?;

            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(keypair.insecure_clone()), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let signature = program
                .request()
                .instruction(solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
                .accounts(validator_blacklist::accounts::VoteAdd {
                    config: config_pubkey,
                    stake_pool: stake_pool_pubkey,
                    blacklist: blacklist_pda,
                    vote_add: vote_add_pda,
                    delegation: delegation_pda,
                    authority: keypair.pubkey(),
                    system_program: system_program::id(),
                })
                .args(validator_blacklist::instruction::VoteAdd {
                    validator_identity_address: validator_pubkey,
                    reason,
                })
                .send()?;

            println!("Vote to add transaction sent: {}", signature);
        },
        "base58" => {
            let manager_pubkey = manager.context("Manager pubkey is required when output is base58")?;
            let manager_pubkey = Pubkey::from_str(&manager_pubkey).context("Invalid manager pubkey")?;

            let dummy_keypair = Keypair::new();
            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(dummy_keypair), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let ixs = program
                .request()
                .instruction(solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
                .accounts(validator_blacklist::accounts::VoteAdd {
                    config: config_pubkey,
                    stake_pool: stake_pool_pubkey,
                    blacklist: blacklist_pda,
                    vote_add: vote_add_pda,
                    delegation: delegation_pda,
                    authority: manager_pubkey,
                    system_program: system_program::id(),
                })
                .args(validator_blacklist::instruction::VoteAdd {
                    validator_identity_address: validator_pubkey,
                    reason,
                })
                .instructions()?;

            let tx = Transaction::new_with_payer(&ixs, Some(&manager_pubkey));
            let serialized = bincode::serialize(&tx).context("Failed to serialize transaction")?;
            let base58_tx = bs58::encode(serialized).into_string();

            println!("{}", base58_tx);
        },
        _ => {
            return Err(anyhow::anyhow!("Invalid output format. Use 'execute' or 'base58'"));
        }
    }

    Ok(())
}

fn handle_vote_remove_command(rpc_url: &str, program_id: &Pubkey, config: String, validator_address: String, stake_pool: String, reason: String, delegation: Option<String>, output: String, manager: Option<String>, keypair_option: Option<String>) -> Result<()> {
    let config_pubkey = Pubkey::from_str(&config).context("Invalid config address")?;
    let validator_pubkey = Pubkey::from_str(&validator_address).context("Invalid validator address")?;
    let stake_pool_pubkey = Pubkey::from_str(&stake_pool).context("Invalid stake pool address")?;
    let delegation_pubkey = if let Some(del) = delegation {
        Some(Pubkey::from_str(&del).context("Invalid delegation address")?)
    } else {
        None
    };

    let (blacklist_pda, _) = Pubkey::find_program_address(
        &[b"blacklist", config_pubkey.as_ref(), validator_pubkey.as_ref()],
        program_id,
    );

    let (vote_remove_pda, _) = Pubkey::find_program_address(
        &[b"vote_remove", config_pubkey.as_ref(), stake_pool_pubkey.as_ref(), validator_pubkey.as_ref()],
        program_id,
    );

    let delegation_pda = delegation_pubkey.map(|_| {
        Pubkey::find_program_address(
            &[b"delegation", config_pubkey.as_ref(), stake_pool_pubkey.as_ref()],
            program_id,
        ).0
    });

    match output.as_str() {
        "execute" => {
            let keypair_path = keypair_option.context("Keypair path is required for execute mode")?;
            let keypair = read_keypair_file(&keypair_path)
                .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?;

            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(keypair.insecure_clone()), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let signature = program
                .request()
                .instruction(solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
                .accounts(validator_blacklist::accounts::VoteRemove {
                    config: config_pubkey,
                    stake_pool: stake_pool_pubkey,
                    blacklist: blacklist_pda,
                    vote_remove: vote_remove_pda,
                    delegation: delegation_pda,
                    authority: keypair.pubkey(),
                    system_program: system_program::id(),
                })
                .args(validator_blacklist::instruction::VoteRemove {
                    validator_identity_address: validator_pubkey,
                    reason,
                })
                .send()?;

            println!("Vote to remove transaction sent: {}", signature);
        },
        "base58" => {
            let manager_pubkey = manager.context("Manager pubkey is required when output is base58")?;
            let manager_pubkey = Pubkey::from_str(&manager_pubkey).context("Invalid manager pubkey")?;

            let dummy_keypair = Keypair::new();
            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(dummy_keypair), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let ixs = program
                .request()
                .instruction(solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
                .accounts(validator_blacklist::accounts::VoteRemove {
                    config: config_pubkey,
                    stake_pool: stake_pool_pubkey,
                    blacklist: blacklist_pda,
                    vote_remove: vote_remove_pda,
                    delegation: delegation_pda,
                    authority: manager_pubkey,
                    system_program: system_program::id(),
                })
                .args(validator_blacklist::instruction::VoteRemove {
                    validator_identity_address: validator_pubkey,
                    reason,
                })
                .instructions()?;

            let tx = Transaction::new_with_payer(&ixs, Some(&manager_pubkey));
            let serialized = bincode::serialize(&tx).context("Failed to serialize transaction")?;
            let base58_tx = bs58::encode(serialized).into_string();

            println!("{}", base58_tx);
        },
        _ => {
            return Err(anyhow::anyhow!("Invalid output format. Use 'execute' or 'base58'"));
        }
    }

    Ok(())
}

fn handle_unvote_add_command(rpc_url: &str, program_id: &Pubkey, config: String, validator_address: String, stake_pool: String, delegation: Option<String>, output: String, manager: Option<String>, keypair_option: Option<String>) -> Result<()> {
    let config_pubkey = Pubkey::from_str(&config).context("Invalid config address")?;
    let validator_pubkey = Pubkey::from_str(&validator_address).context("Invalid validator address")?;
    let stake_pool_pubkey = Pubkey::from_str(&stake_pool).context("Invalid stake pool address")?;
    let delegation_pubkey = if let Some(del) = delegation {
        Some(Pubkey::from_str(&del).context("Invalid delegation address")?)
    } else {
        None
    };

    let (blacklist_pda, _) = Pubkey::find_program_address(
        &[b"blacklist", config_pubkey.as_ref(), validator_pubkey.as_ref()],
        program_id,
    );

    let (vote_add_pda, _) = Pubkey::find_program_address(
        &[b"vote_add", config_pubkey.as_ref(), stake_pool_pubkey.as_ref(), validator_pubkey.as_ref()],
        program_id,
    );

    let delegation_pda = delegation_pubkey.map(|_| {
        Pubkey::find_program_address(
            &[b"delegation", config_pubkey.as_ref(), stake_pool_pubkey.as_ref()],
            program_id,
        ).0
    });

    match output.as_str() {
        "execute" => {
            let keypair_path = keypair_option.context("Keypair path is required for execute mode")?;
            let keypair = read_keypair_file(&keypair_path)
                .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?;

            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(keypair.insecure_clone()), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let signature = program
                .request()
                .instruction(solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
                .accounts(validator_blacklist::accounts::UnvoteAdd {
                    config: config_pubkey,
                    stake_pool: stake_pool_pubkey,
                    blacklist: blacklist_pda,
                    vote_add: vote_add_pda,
                    delegation: delegation_pda,
                    authority: keypair.pubkey(),
                })
                .args(validator_blacklist::instruction::UnvoteAdd {
                    validator_identity_address: validator_pubkey,
                })
                .send()?;

            println!("Unvote add transaction sent: {}", signature);
        },
        "base58" => {
            let manager_pubkey = manager.context("Manager pubkey is required when output is base58")?;
            let manager_pubkey = Pubkey::from_str(&manager_pubkey).context("Invalid manager pubkey")?;

            let dummy_keypair = Keypair::new();
            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(dummy_keypair), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let ixs = program
                .request()
                .instruction(solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
                .accounts(validator_blacklist::accounts::UnvoteAdd {
                    config: config_pubkey,
                    stake_pool: stake_pool_pubkey,
                    blacklist: blacklist_pda,
                    vote_add: vote_add_pda,
                    delegation: delegation_pda,
                    authority: manager_pubkey,
                })
                .args(validator_blacklist::instruction::UnvoteAdd {
                    validator_identity_address: validator_pubkey,
                })
                .instructions()?;

            let tx = Transaction::new_with_payer(&ixs, Some(&manager_pubkey));
            let serialized = bincode::serialize(&tx).context("Failed to serialize transaction")?;
            let base58_tx = bs58::encode(serialized).into_string();

            println!("{}", base58_tx);
        },
        _ => {
            return Err(anyhow::anyhow!("Invalid output format. Use 'execute' or 'base58'"));
        }
    }

    Ok(())
}

fn handle_unvote_remove_command(rpc_url: &str, program_id: &Pubkey, config: String, validator_address: String, stake_pool: String, delegation: Option<String>, output: String, manager: Option<String>, keypair_option: Option<String>) -> Result<()> {
    let config_pubkey = Pubkey::from_str(&config).context("Invalid config address")?;
    let validator_pubkey = Pubkey::from_str(&validator_address).context("Invalid validator address")?;
    let stake_pool_pubkey = Pubkey::from_str(&stake_pool).context("Invalid stake pool address")?;
    let delegation_pubkey = if let Some(del) = delegation {
        Some(Pubkey::from_str(&del).context("Invalid delegation address")?)
    } else {
        None
    };

    let (blacklist_pda, _) = Pubkey::find_program_address(
        &[b"blacklist", config_pubkey.as_ref(), validator_pubkey.as_ref()],
        program_id,
    );

    let (vote_remove_pda, _) = Pubkey::find_program_address(
        &[b"vote_remove", config_pubkey.as_ref(), stake_pool_pubkey.as_ref(), validator_pubkey.as_ref()],
        program_id,
    );

    let delegation_pda = delegation_pubkey.map(|_| {
        Pubkey::find_program_address(
            &[b"delegation", config_pubkey.as_ref(), stake_pool_pubkey.as_ref()],
            program_id,
        ).0
    });

    match output.as_str() {
        "execute" => {
            let keypair_path = keypair_option.context("Keypair path is required for execute mode")?;
            let keypair = read_keypair_file(&keypair_path)
                .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?;

            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(keypair.insecure_clone()), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let signature = program
                .request()
                .instruction(solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
                .accounts(validator_blacklist::accounts::UnvoteRemove {
                    config: config_pubkey,
                    stake_pool: stake_pool_pubkey,
                    blacklist: blacklist_pda,
                    vote_remove: vote_remove_pda,
                    delegation: delegation_pda,
                    authority: keypair.pubkey(),
                })
                .args(validator_blacklist::instruction::UnvoteRemove {
                    validator_identity_address: validator_pubkey,
                })
                .send()?;

            println!("Unvote remove transaction sent: {}", signature);
        },
        "base58" => {
            let manager_pubkey = manager.context("Manager pubkey is required when output is base58")?;
            let manager_pubkey = Pubkey::from_str(&manager_pubkey).context("Invalid manager pubkey")?;

            let dummy_keypair = Keypair::new();
            let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
            let client = Client::new_with_options(cluster, Rc::new(dummy_keypair), CommitmentConfig::confirmed());
            let program = client.program(*program_id)?;

            let ixs = program
                .request()
                .instruction(solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
                .accounts(validator_blacklist::accounts::UnvoteRemove {
                    config: config_pubkey,
                    stake_pool: stake_pool_pubkey,
                    blacklist: blacklist_pda,
                    vote_remove: vote_remove_pda,
                    delegation: delegation_pda,
                    authority: manager_pubkey,
                })
                .args(validator_blacklist::instruction::UnvoteRemove {
                    validator_identity_address: validator_pubkey,
                })
                .instructions()?;

            let tx = Transaction::new_with_payer(&ixs, Some(&manager_pubkey));
            let serialized = bincode::serialize(&tx).context("Failed to serialize transaction")?;
            let base58_tx = bs58::encode(serialized).into_string();

            println!("{}", base58_tx);
        },
        _ => {
            return Err(anyhow::anyhow!("Invalid output format. Use 'execute' or 'base58'"));
        }
    }

    Ok(())
}
