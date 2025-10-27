use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "validator-blacklist-cli")]
#[command(about = "A CLI for managing validator blacklists")]
pub struct Cli {
    #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
    pub rpc: String,

    #[arg(short, long)]
    pub keypair: Option<String>,

    #[arg(short, long, default_value = "VBLCKLiST8oNqfG3UKvWKJGJdunEDCqCxmgJJvP9dFp")]
    pub program_id: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all blacklisted validators
    List,
    
    /// Create a new config account
    CreateConfig {
        #[arg(short, long)]
        config: String,
        #[arg(short, long)]
        min_tvl: u64,
        #[arg(short, long, value_delimiter = ',')]
        allowed_programs: Vec<String>,
        #[arg(short = 'o', long, default_value = "execute")]
        output: String,
        #[arg(short = 'M', long)]
        manager: Option<String>,
    },
    
    /// Update an existing config account
    UpdateConfig {
        #[arg(short, long)]
        config: String,
        #[arg(short, long)]
        min_tvl: Option<u64>,
        #[arg(short, long, value_delimiter = ',')]
        allowed_programs: Option<Vec<String>>,
        #[arg(short = 'o', long, default_value = "execute")]
        output: String,
        #[arg(short = 'M', long)]
        manager: Option<String>,
    },
    
    /// Update config admin
    UpdateConfigAdmin {
        #[arg(short, long)]
        config: String,
        #[arg(short, long)]
        new_admin: String,
        #[arg(short = 'o', long, default_value = "execute")]
        output: String,
        #[arg(short = 'M', long)]
        manager: Option<String>,
    },
    
    /// Vote to add a validator to the blacklist
    VoteAdd {
        #[arg(short, long)]
        config: String,
        #[arg(short, long)]
        validator_address: String,
        #[arg(short, long)]
        stake_pool: String,
        #[arg(short, long)]
        reason: String,
        #[arg(short, long)]
        delegation: Option<String>,
        #[arg(short = 'o', long, default_value = "execute")]
        output: String,
        #[arg(short = 'M', long)]
        manager: Option<String>,
    },
    
    /// Vote to remove a validator from the blacklist
    VoteRemove {
        #[arg(short, long)]
        config: String,
        #[arg(short, long)]
        validator_address: String,
        #[arg(short, long)]
        stake_pool: String,
        #[arg(short, long)]
        reason: String,
        #[arg(short, long)]
        delegation: Option<String>,
        #[arg(short = 'o', long, default_value = "execute")]
        output: String,
        #[arg(short = 'M', long)]
        manager: Option<String>,
    },
    
    /// Unvote add (remove a previous add vote)
    UnvoteAdd {
        #[arg(short, long)]
        config: String,
        #[arg(short, long)]
        validator_address: String,
        #[arg(short, long)]
        stake_pool: String,
        #[arg(short, long)]
        delegation: Option<String>,
        #[arg(short = 'o', long, default_value = "execute")]
        output: String,
        #[arg(short = 'M', long)]
        manager: Option<String>,
    },
    
    /// Unvote remove (remove a previous remove vote)
    UnvoteRemove {
        #[arg(short, long)]
        config: String,
        #[arg(short, long)]
        validator_address: String,
        #[arg(short, long)]
        stake_pool: String,
        #[arg(short, long)]
        delegation: Option<String>,
        #[arg(short = 'o', long, default_value = "execute")]
        output: String,
        #[arg(short = 'M', long)]
        manager: Option<String>,
    },
    
    /// Delegate authority to another account
    Delegate {
        #[arg(short, long)]
        config: String,
        #[arg(short, long)]
        stake_pool: String,
        #[arg(short, long)]
        delegate: String,
        #[arg(short = 'o', long, default_value = "execute")]
        output: String,
        #[arg(short = 'M', long)]
        manager: Option<String>,
    },
    
    /// Remove delegation (undelegate)
    Undelegate {
        #[arg(short, long)]
        config: String,
        #[arg(short, long)]
        stake_pool: String,
        #[arg(short = 'o', long, default_value = "execute")]
        output: String,
        #[arg(short = 'M', long)]
        manager: Option<String>,
    },
}
