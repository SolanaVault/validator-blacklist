use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "validator-blacklist-cli")]
#[command(about = "A CLI tool for interacting with the Solana validator blacklist program")]

pub struct Cli {
    /// RPC URL for Solana cluster
    #[arg(short('u'), long, default_value = "http://localhost:8899")]
    pub rpc: String,

    /// Program ID of the validator blacklist program
    #[arg(short, long)]
    pub program_id: String,

    /// Keypair file path for the authority
    #[arg(short, long)]
    pub keypair: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all blacklisted validators and their vote tallies
    List,
    
    /// Create the global configuration account
    CreateConfig {
        /// Config account address (should be a keypair)
        config: String,
        /// Minimum TVL required for stake pools
        min_tvl: u64,
        /// Comma-separated list of allowed stake pool program IDs
        #[arg(value_delimiter = ',')]
        allowed_programs: Vec<String>,
    },
    
    /// Update the config settings (min_tvl and/or allowed_programs)
    UpdateConfig {
        /// Config account address
        config: String,
        /// New minimum TVL (optional)
        #[arg(long)]
        min_tvl: Option<u64>,
        /// Comma-separated list of new allowed stake pool program IDs (optional)
        #[arg(long, value_delimiter = ',')]
        allowed_programs: Option<Vec<String>>,
    },
    
    /// Update the admin of the config
    UpdateConfigAdmin {
        /// Config account address
        config: String,
        /// New admin pubkey
        new_admin: String,
    },
    
    /// Vote to add a validator to the blacklist
    VoteAdd {
        /// Config account address
        config: String,
        /// Validator identity address to blacklist
        validator_address: String,
        /// Stake pool address casting the vote
        stake_pool: String,
        /// Reason for blacklisting
        reason: String,
        /// Optional delegation address if using delegated authority
        #[arg(long)]
        delegation: Option<String>,
    },
    
    /// Vote to remove a validator from the blacklist
    VoteRemove {
        /// Config account address
        config: String,
        /// Validator identity address to remove from blacklist
        validator_address: String,
        /// Stake pool address casting the vote
        stake_pool: String,
        /// Reason for removal
        reason: String,
        /// Optional delegation address if using delegated authority
        #[arg(long)]
        delegation: Option<String>,
    },
    
    /// Remove a previously cast vote to add a validator
    UnvoteAdd {
        /// Config account address
        config: String,
        /// Validator identity address
        validator_address: String,
        /// Stake pool address that cast the original vote
        stake_pool: String,
        /// Optional delegation address if using delegated authority
        #[arg(long)]
        delegation: Option<String>,
    },
    
    /// Remove a previously cast vote to remove a validator
    UnvoteRemove {
        /// Config account address
        config: String,
        /// Validator identity address
        validator_address: String,
        /// Stake pool address that cast the original vote
        stake_pool: String,
        /// Optional delegation address if using delegated authority
        #[arg(long)]
        delegation: Option<String>,
    },
    
    /// Create a delegation from stake pool manager to another authority
    Delegate {
        /// Config account address
        config: String,
        /// Stake pool address
        stake_pool: String,
        /// Address to delegate authority to
        delegate: String,
        /// Output format: 'execute' (default) to execute the transaction, or 'base58' to serialize and print the transaction in base58 format
        #[arg(long, default_value = "execute", help = "Output format: 'execute' (default) or 'base58'")]
        output: String,
        /// Manager pubkey (required when --output is 'base58'). When using base58 output, provide the manager's public key instead of using a keypair file
        #[arg(long, help = "Manager pubkey (required when output is base58)")]
        manager: Option<String>,
    },
    
    /// Remove a delegation
    Undelegate {
        /// Config account address
        config: String,
        /// Stake pool address
        stake_pool: String,
        /// Output format: 'execute' (default) to execute the transaction, or 'base58' to serialize and print the transaction in base58 format
        #[arg(long, default_value = "execute", help = "Output format: 'execute' (default) or 'base58'")]
        output: String,
        /// Manager pubkey (required when --output is 'base58'). When using base58 output, provide the manager's public key instead of using a keypair file
        #[arg(long, help = "Manager pubkey (required when output is base58)")]
        manager: Option<String>,
    },
}