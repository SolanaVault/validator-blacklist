use crate::cli::{Cli, Commands};
use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, read_keypair_file},
    signer::Signer,
};
use solana_commitment_config::CommitmentConfig;
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
        Commands::CreateConfig { config, min_tvl, allowed_programs } => {
            handle_create_config_command(&cli.rpc, &program_id, config, min_tvl, allowed_programs, cli.keypair)?;
        }
        Commands::UpdateConfig { config, min_tvl, allowed_programs } => {
            handle_update_config_command(&cli.rpc, &program_id, config, min_tvl, allowed_programs, cli.keypair)?;
        }
        Commands::UpdateConfigAdmin { config, new_admin } => {
            handle_update_config_admin_command(&cli.rpc, &program_id, config, new_admin, cli.keypair)?;
        }
        Commands::VoteAdd { config, validator_address, stake_pool, reason, delegation } => {
            handle_vote_add_command(&cli.rpc, &program_id, config, validator_address, stake_pool, reason, delegation, cli.keypair)?;
        }
        Commands::VoteRemove { config, validator_address, stake_pool, reason, delegation } => {
            handle_vote_remove_command(&cli.rpc, &program_id, config, validator_address, stake_pool, reason, delegation, cli.keypair)?;
        }
        Commands::UnvoteAdd { config, validator_address, stake_pool, delegation } => {
            handle_unvote_add_command(&cli.rpc, &program_id, config, validator_address, stake_pool, delegation, cli.keypair)?;
        }
        Commands::UnvoteRemove { config, validator_address, stake_pool, delegation } => {
            handle_unvote_remove_command(&cli.rpc, &program_id, config, validator_address, stake_pool, delegation, cli.keypair)?;
        }
        Commands::BatchBan { config, stake_pool, file, validators_file, delegation } => {
            handle_batch_ban_command(&cli.rpc, &program_id, config, stake_pool, file, validators_file, delegation, cli.keypair)?;
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
        }
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
            println!("{}", base58_tx);
        }
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
        }
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
        }
        _ => {
            return Err(anyhow::anyhow!("Invalid output format. Use 'execute' or 'base58'"));
        }
    }

    Ok(())
}

fn handle_create_config_command(rpc_url: &str, program_id: &Pubkey, config: String, min_tvl: u64, allowed_programs: Vec<String>, keypair_option: Option<String>) -> Result<()> {
    let allowed_program_pubkeys: Result<Vec<Pubkey>> = allowed_programs
        .iter()
        .map(|p| Pubkey::from_str(p).context(format!("Invalid program address: {}", p)))
        .collect();
    let allowed_program_pubkeys = allowed_program_pubkeys?;

    let keypair_path = keypair_option.context("Keypair path is required")?;
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

    Ok(())
}

fn handle_update_config_command(rpc_url: &str, program_id: &Pubkey, config: String, min_tvl: Option<u64>, allowed_programs: Option<Vec<String>>, keypair_option: Option<String>) -> Result<()> {
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

    let keypair_path = keypair_option.context("Keypair path is required")?;
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

    Ok(())
}

fn handle_update_config_admin_command(rpc_url: &str, program_id: &Pubkey, config: String, new_admin: String, keypair_option: Option<String>) -> Result<()> {
    let config_pubkey = Pubkey::from_str(&config).context("Invalid config address")?;
    let new_admin_pubkey = Pubkey::from_str(&new_admin).context("Invalid new admin address")?;

    let keypair_path = keypair_option.context("Keypair path is required")?;
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

    Ok(())
}

fn handle_vote_add_command(rpc_url: &str, program_id: &Pubkey, config: String, validator_address: String, stake_pool: String, reason: String, delegation: Option<String>, keypair_option: Option<String>) -> Result<()> {
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

    let keypair_path = keypair_option.context("Keypair path is required")?;
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

    Ok(())
}

fn handle_vote_remove_command(rpc_url: &str, program_id: &Pubkey, config: String, validator_address: String, stake_pool: String, reason: String, delegation: Option<String>, keypair_option: Option<String>) -> Result<()> {
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

    let keypair_path = keypair_option.context("Keypair path is required")?;
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

    Ok(())
}

fn handle_unvote_add_command(rpc_url: &str, program_id: &Pubkey, config: String, validator_address: String, stake_pool: String, delegation: Option<String>, keypair_option: Option<String>) -> Result<()> {
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

    let keypair_path = keypair_option.context("Keypair path is required")?;
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

    Ok(())
}

fn handle_unvote_remove_command(rpc_url: &str, program_id: &Pubkey, config: String, validator_address: String, stake_pool: String, delegation: Option<String>, keypair_option: Option<String>) -> Result<()> {
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

    let keypair_path = keypair_option.context("Keypair path is required")?;
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

    Ok(())
}

fn handle_batch_ban_command(rpc_url: &str, program_id: &Pubkey, config: String, stake_pool: String, csv: String, validators_file: Option<String>, delegation: Option<String>, keypair_option: Option<String>) -> Result<()> {
    use std::fs;
    use std::collections::HashSet;

    let config_pubkey = Pubkey::from_str(&config).context("Invalid config address")?;
    let stake_pool_pubkey = Pubkey::from_str(&stake_pool).context("Invalid stake pool address")?;

    // Read the CSV file
    let mut validator_addresses = Vec::new();
    let mut csv_reasons = Vec::new();

    println!("📖 Reading CSV file: {}", csv);
    let mut rdr = csv::Reader::from_path(&csv)?;
    let mut row_count = 0;

    for result in rdr.records() {
        let record = result.context("Invalid CSV record")?;

        // Skip empty lines
        if record.len() == 0 || record.get(0).map(|s| s.is_empty()).unwrap_or(true) {
            continue;
        }

        // Skip header line (check if first field looks like "validator" or similar)
        if row_count == 0 && (record.get(0).unwrap_or(&"").to_lowercase().contains("validator") ||
            record.get(0).unwrap_or(&"").to_lowercase().contains("address")) {
            println!("   ℹ️  Skipping header row");
            continue;
        }

        if record.len() < 1 {
            println!("   ⚠️  Skipping invalid row (expected at least 1 column): {:?}", record);
            continue;
        }

        let validator_address = record.get(0).context("Missing validator_address")?;

        // Reason is required in the second column
        let csv_reason = record.get(1)
            .context(format!("Missing reason on row {}: validator {} has no reason provided", row_count + 1, validator_address))?
            .to_string();

        let validator_pubkey = Pubkey::from_str(validator_address)
            .context(format!("Invalid validator address on row {}: {}", row_count + 1, validator_address))?;

        validator_addresses.push(validator_pubkey);
        csv_reasons.push(csv_reason);
        row_count += 1;
    }

    println!("✅ Loaded {} validators from CSV", validator_addresses.len());

    if validator_addresses.is_empty() {
        return Err(anyhow::anyhow!("No valid validators found in CSV file"));
    }

    // If validators file is provided, filter by active validators
    let active_validators = if let Some(validators_file_path) = validators_file {
        println!("📋 Reading validators list from: {}", validators_file_path);
        let validators_content = fs::read_to_string(&validators_file_path)
            .context(format!("Failed to read validators file: {}", validators_file_path))?;

        let entries = crate::validator_parser::parse_validator_list(&validators_content)?;
        let active_set: HashSet<Pubkey> = entries.iter().map(|e| e.identity).collect();

        println!("✅ Loaded {} active validators from list", active_set.len());
        Some(active_set)
    } else {
        None
    };

    // Filter validators to only those in the active list
    let mut filtered_validators = Vec::new();
    let mut filtered_reasons = Vec::new();
    let mut skipped_count = 0;

    for (validator_pubkey, reason) in validator_addresses.iter().zip(csv_reasons.iter()) {
        if let Some(ref active_set) = active_validators {
            if !active_set.contains(validator_pubkey) {
                println!("⏭️  Skipping {} (not in active validators list)", validator_pubkey);
                skipped_count += 1;
                continue;
            }
        }
        filtered_validators.push(*validator_pubkey);
        filtered_reasons.push(reason.clone());
    }

    if skipped_count > 0 {
        println!("⏭️  Skipped {} validators (already shut down)", skipped_count);
    }

    if filtered_validators.is_empty() {
        return Err(anyhow::anyhow!("No active validators to ban after filtering"));
    }

    println!("🎯 Will ban {} validators\n", filtered_validators.len());

    let delegation_pubkey = delegation.as_ref()
        .map(|del| Pubkey::from_str(del).context("Invalid delegation address"))
        .transpose()?;

    let delegation_pda = delegation_pubkey.as_ref().map(|_| {
        Pubkey::find_program_address(
            &[b"delegation", config_pubkey.as_ref(), stake_pool_pubkey.as_ref()],
            program_id,
        ).0
    });

    let keypair_path = keypair_option.context("Keypair path is required for execute mode")?;
    let keypair = read_keypair_file(&keypair_path)
        .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?;

    let cluster = Cluster::Custom(rpc_url.to_string(), "none".to_string());
    let client = Client::new_with_options(cluster, Rc::new(keypair.insecure_clone()), CommitmentConfig::confirmed());
    let program = client.program(*program_id)?;

    println!("Starting batch ban...\n");

    for (i, (validator_pubkey, ban_reason)) in filtered_validators.iter().zip(filtered_reasons.iter()).enumerate() {
        let (blacklist_pda, _) = Pubkey::find_program_address(
            &[b"blacklist", config_pubkey.as_ref(), validator_pubkey.as_ref()],
            program_id,
        );

        let (vote_add_pda, _) = Pubkey::find_program_address(
            &[b"vote_add", config_pubkey.as_ref(), stake_pool_pubkey.as_ref(), validator_pubkey.as_ref()],
            program_id,
        );

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
                validator_identity_address: *validator_pubkey,
                reason: ban_reason.clone(),
            })
            .send()?;

        println!("[{}/{}] ✓ Voted to ban validator {} for reason: \"{}\"",
                 i + 1, filtered_validators.len(), validator_pubkey, ban_reason);
        println!("        Transaction signature: {}", signature);
    }

    println!("\n✅ Batch ban completed successfully!");
    Ok(())
}
