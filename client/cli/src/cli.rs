use clap::{Parser, Subcommand};

const DEFAULT_CONFIG: &str = "8wXtPM3EHPu4BKXpBCrWXqhzPc9vS2HSkD9veATmU4Yq";
const DEFAULT_PROGRAM_ID: &str = "Fu4zvEKjgxWjaQifp7fyghKJfk6HzUCaJRvoGffJBm6Q";

#[derive(Parser)]
#[command(name = "validator-blacklist-cli")]
#[command(about = "A CLI for managing validator blacklists")]
pub struct Cli {
    #[arg(short, long, default_value = "http://localhost:8899")]
    pub rpc: String,

    #[arg(short, long)]
    pub keypair: Option<String>,

    #[arg(short, long, default_value = DEFAULT_PROGRAM_ID)]
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
        #[arg(short, long, default_value = DEFAULT_CONFIG)]
        config: String,
        #[arg(short, long)]
        min_tvl: u64,
        #[arg(short, long, value_delimiter = ',')]
        allowed_programs: Vec<String>,
    },
    
    /// Update an existing config account
    UpdateConfig {
        #[arg(short, long, default_value = DEFAULT_CONFIG)]
        config: String,
        #[arg(short, long)]
        min_tvl: Option<u64>,
        #[arg(short, long, value_delimiter = ',')]
        allowed_programs: Option<Vec<String>>,
    },
    
    /// Update config admin
    UpdateConfigAdmin {
        #[arg(short, long, default_value = DEFAULT_CONFIG)]
        config: String,
        #[arg(short, long)]
        new_admin: String,
    },
    
    /// Vote to add a validator to the blacklist
    VoteAdd {
        #[arg(short, long, default_value = DEFAULT_CONFIG)]
        config: String,
        #[arg(short, long)]
        validator_address: String,
        #[arg(short, long)]
        stake_pool: String,
        #[arg(short, long)]
        reason: String,
        #[arg(short, long)]
        delegation: Option<String>,
    },
    
    /// Vote to remove a validator from the blacklist
    VoteRemove {
        #[arg(short, long, default_value = DEFAULT_CONFIG)]
        config: String,
        #[arg(short, long)]
        validator_address: String,
        #[arg(short, long)]
        stake_pool: String,
        #[arg(short, long)]
        reason: String,
        #[arg(short, long)]
        delegation: Option<String>,
    },
    
    /// Unvote add (remove a previous add vote)
    UnvoteAdd {
        #[arg(short, long, default_value = DEFAULT_CONFIG)]
        config: String,
        #[arg(short, long)]
        validator_address: String,
        #[arg(short, long)]
        stake_pool: String,
        #[arg(short, long)]
        delegation: Option<String>,
    },
    
    /// Unvote remove (remove a previous remove vote)
    UnvoteRemove {
        #[arg(short, long, default_value = DEFAULT_CONFIG)]
        config: String,
        #[arg(short, long)]
        validator_address: String,
        #[arg(short, long)]
        stake_pool: String,
        #[arg(short, long)]
        delegation: Option<String>,
    },
    
    /// Delegate authority to another account
    Delegate {
        #[arg(short, long, default_value = DEFAULT_CONFIG)]
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
        #[arg(short, long, default_value = DEFAULT_CONFIG)]
        config: String,
        #[arg(short, long)]
        stake_pool: String,
        #[arg(short = 'o', long, default_value = "execute")]
        output: String,
        #[arg(short = 'M', long)]
        manager: Option<String>,
    },

    /// Batch ban validators from a CSV file, filtering by active validators
    BatchBan {
        #[arg(short, long, default_value = DEFAULT_CONFIG)]
        config: String,
        #[arg(short, long)]
        stake_pool: String,
        #[arg(short = 'f', long)]
        file: String,
        #[arg(short = 'v', long)]
        validators_file: Option<String>,
        #[arg(short, long)]
        delegation: Option<String>,
    },
}
