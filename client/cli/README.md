# Validator Blacklist CLI

A command-line interface for interacting with the Solana Validator Blacklist program.

## Installation

1. Make sure you have Rust installed
2. Navigate to the CLI directory:
   ```bash
   cd client/cli
   ```
3. Build the CLI:
   ```bash
   cargo build --release
   ```

## Usage

### Basic Syntax

```bash
validator-blacklist-cli [OPTIONS] <COMMAND>
```

### Options

- `-r, --rpc-url <RPC_URL>`: RPC URL for Solana cluster (default: https://api.mainnet-beta.solana.com)
- `-p, --program-id <PROGRAM_ID>`: Program ID of the validator blacklist program (required)
- `-k, --keypair <KEYPAIR>`: Keypair file path for the authority (required for most commands)

### Commands

#### List Blacklisted Validators

List all validators currently on the blacklist with their vote tallies:

```bash
validator-blacklist-cli -p <PROGRAM_ID> list
```

#### Vote to Add a Validator

Cast a vote to add a validator to the blacklist:

```bash
validator-blacklist-cli -p <PROGRAM_ID> -k <KEYPAIR> vote-add <VALIDATOR_ADDRESS> <STAKE_POOL> "<REASON>"
```

With delegation:
```bash
validator-blacklist-cli -p <PROGRAM_ID> -k <KEYPAIR> vote-add <VALIDATOR_ADDRESS> <STAKE_POOL> "<REASON>" --delegation <DELEGATION_ADDRESS>
```

#### Vote to Remove a Validator

Cast a vote to remove a validator from the blacklist:

```bash
validator-blacklist-cli -p <PROGRAM_ID> -k <KEYPAIR> vote-remove <VALIDATOR_ADDRESS> <STAKE_POOL> "<REASON>"
```

#### Remove a Vote to Add

Remove a previously cast vote to add a validator:

```bash
validator-blacklist-cli -p <PROGRAM_ID> -k <KEYPAIR> unvote-add <VALIDATOR_ADDRESS> <STAKE_POOL>
```

#### Remove a Vote to Remove

Remove a previously cast vote to remove a validator:

```bash
validator-blacklist-cli -p <PROGRAM_ID> -k <KEYPAIR> unvote-remove <VALIDATOR_ADDRESS> <STAKE_POOL>
```

#### Create Delegation

Delegate authority from a stake pool manager to another address:

```bash
validator-blacklist-cli -p <PROGRAM_ID> -k <KEYPAIR> delegate <STAKE_POOL> <DELEGATE_ADDRESS>
```

#### Remove Delegation

Remove a delegation:

```bash
validator-blacklist-cli -p <PROGRAM_ID> -k <KEYPAIR> undelegate <STAKE_POOL>
```

## Examples

### List all blacklisted validators on devnet:

```bash
validator-blacklist-cli \
  --rpc-url https://api.devnet.solana.com \
  --program-id YourProgramIdHere \
  list
```

### Vote to blacklist a validator:

```bash
validator-blacklist-cli \
  --rpc-url https://api.devnet.solana.com \
  --program-id YourProgramIdHere \
  --keypair ~/.config/solana/id.json \
  vote-add \
  ValidatorIdentityAddressHere \
  StakePoolAddressHere \
  "Validator showing suspicious behavior"
```

### Create a delegation:

```bash
validator-blacklist-cli \
  --rpc-url https://api.devnet.solana.com \
  --program-id YourProgramIdHere \
  --keypair ~/.config/solana/id.json \
  delegate \
  StakePoolAddressHere \
  DelegateAddressHere
```
