## Batch Ban Feature with Active Validator Filtering

This feature allows you to batch ban validators from a CSV file, but only bans those that are still active (preventing wasted transactions on already-shut-down validators).

### Usage

#### Basic usage (ban all validators in CSV):
```bash
validator-blacklist-cli batch-ban \
  --config <CONFIG_ADDRESS> \
  --stake-pool <STAKE_POOL_ADDRESS> \
  --file validators_to_ban.csv \
  --keypair /path/to/keypair.json
```

#### Advanced usage (filter by active validators):
```bash
# First, get the active validators list from solana:
solana validators > active_validators.txt

# Then run batch ban with filtering:
validator-blacklist-cli batch-ban \
  --config <CONFIG_ADDRESS> \
  --stake-pool <STAKE_POOL_ADDRESS> \
  --file validators_to_ban.csv \
  --validators-file active_validators.txt \
  --keypair /path/to/keypair.json
```

### CSV Format

The CSV file should contain validator identity addresses with ban reasons (both columns are required):

```csv
validator_address,reason
3iQqh65Gby53aaYUF8ocoiEyhBs4aoe7BTYYWvy1c9dF,Policy violation
BULKzD8ZgbYV6taZjXYkdSytcutscMGTFFi2MDHViKdc,Spam activity
```

Each row must have both:
1. **validator_address**: The identity public key of the validator (not vote key)
2. **reason**: The ban reason (required - cannot be empty)

### Validators List Format

The `validators_file` should be the output from `solana validators get`. The parser handles:
- Header rows
- Emoji prefixes (⚠️) before validator identities
- Multiple whitespace-separated columns
- Empty lines

Example format:
```
   Identity                                      Vote Account                            Commission  Last Vote
  3iQqh65Gby53aaYUF8ocoiEyhBs4aoe7BTYYWvy1c9dF  AEtdq4CwtuktCEUWLLpRTNPBZs6tr7BBqxkHJ1DjAttR    5%  379088558
⚠️2XK1YYuLwPCMZSbmfedmso1vmkqrX63M2srNApvAntvw  ENjAU1VZvBTAMCwg9ZayfxLaRQEPExcR2ujH7VdeBkDh  100%  378433337
```

### How It Works

1. **Reads CSV file**: Parses all validator addresses and reasons from the CSV
2. **Reads validators list** (if provided): Parses active validators from `solana validators get` output
3. **Filters**: Only includes validators that are in the active list
4. **Reports**: Shows which validators are skipped (already shut down)
5. **Executes**: Sends vote-add transactions for all active validators

### Example Output

```
📖 Reading CSV file: validators_to_ban.csv
   ℹ️  Skipping header row
✅ Loaded 3 validators from CSV
📋 Reading validators list from: active_validators.txt
✅ Loaded 450 active validators from list
⏭️  Skipping 3iQqh65Gby53aaYUF8ocoiEyhBs4aoe7BTYYWvy1c9dF (not in active validators list)
🎯 Will ban 2 validators

Starting batch ban...

[1/2] ✓ Voted to ban validator BULKzD8ZgbYV6taZjXYkdSytcutscMGTFFi2MDHViKdc for reason: "Spam activity"
        Transaction signature: 5bEj...
[2/2] ✓ Voted to ban validator 9J11DedXf8LKA6mE3fXLAXkdoQPa1r2E8pfE3iZ5UWwT for reason: "Policy violation"
        Transaction signature: 3xKj...

✅ Batch ban completed successfully!
```

### Optional Parameters

- `--delegation <DELEGATION_ADDRESS>`: Specify a delegation PDA if needed
- `--validators-file <FILE>`: Filter by active validators (optional)

### Command Parameters

```
USAGE:
    validator-blacklist-cli batch-ban [OPTIONS] --stake-pool <STAKE_POOL>

OPTIONS:
  -c, --config <CONFIG>                 Config account address [default: 8wXtPM3EHPu4BKXpBCrWXqhzPc9vS2HSkD9veATmU4Yq]
  -s, --stake-pool <STAKE_POOL>         Stake pool address [required]
  -f, --file <FILE>                     CSV file with validators to ban [required]
  -v, --validators-file <FILE>          Output from 'solana validators get' for filtering [optional]
  --delegation <DELEGATION>             Delegation PDA address [optional]
  -k, --keypair <KEYPAIR>               Path to keypair file [optional]
  -p, --program-id <PROGRAM_ID>         Program ID [default: Fu4zvEKjgxWjaQifp7fyghKJfk6HzUCaJRvoGffJBm6Q]
```
