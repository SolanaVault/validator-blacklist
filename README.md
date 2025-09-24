# Validator Blacklist

An on-chain program that manages a decentralized database of blacklisted validators that have been classified as malicious by one or more stake pool operators. Stake pool operators represent a significant part of the stake distribution on Solana, and as such have a significant responsibility to not delegate stake to validators that harm the network and/or UX of interacting with the Solana network.

Program is deployed by The Vault on mainnet at Fu4zvEKjgxWjaQifp7fyghKJfk6HzUCaJRvoGffJBm6Q

## Overview

This program allows stake pool operators to vote on adding or removing validators from a blacklist. The voting system is transparent and auditable, with all votes stored on-chain. Stake pool operators use their SPL stake pool manager key as authority to vote (or delegate the authority to another key for convenience).

## Actors

### Stake Pool Operators
- Can vote to add or remove validators from the blacklist
- Identify themselves using their stake pool manager key
- Can delegate their voting authority to another address
- Can revoke delegations they have previously created

### Viewers
- General users who can view the blacklist and vote history
- Use `getProgramAccounts` to enumerate blacklisted validators and votes

## State Accounts

### Blacklist
- **Purpose**: Tracks a validator that has received votes for blacklisting
- **Seed**: `["blacklist", validator_identity_address]`
- **Fields**:
  - `validator_identity_address`: The validator's public key
  - `tally_add`: Number of votes to add to blacklist
  - `tally_remove`: Number of votes to remove from blacklist

### VoteAddToBlacklist
- **Purpose**: Records a vote to add a validator to the blacklist
- **Seed**: `["vote_add", operator_pubkey, validator_identity_address]`
- **Fields**:
  - `operator`: The operator who cast the vote
  - `validator_identity_address`: The validator being voted on
  - `reason`: Explanation for the vote (max 200 chars)
  - `timestamp`: UTC timestamp when vote was cast
  - `slot`: Solana slot when vote was cast

### VoteRemoveFromBlacklist
- **Purpose**: Records a vote to remove a validator from the blacklist
- **Seed**: `["vote_remove", operator_pubkey, validator_identity_address]`
- **Fields**:
  - `operator`: The operator who cast the vote
  - `validator_identity_address`: The validator being voted on
  - `reason`: Explanation for the vote (max 200 chars)
  - `timestamp`: UTC timestamp when vote was cast
  - `slot`: Solana slot when vote was cast

### Delegation
- **Purpose**: Records delegation of voting authority from a stake pool manager to another address
- **Seed**: `["delegation", stake_pool_address, manager_pubkey]`
- **Fields**:
  - `stake_pool`: The stake pool address
  - `manager`: The original stake pool manager
  - `delegate`: The address that has been delegated authority
  - `timestamp`: UTC timestamp when delegation was created

## Instructions

### vote_add
- **Purpose**: Vote to add a validator to the blacklist
- **Parameters**:
  - `validator_identity_address`: The validator to blacklist
  - `reason`: Explanation for the vote
- **Behavior**: Creates/updates Blacklist account and creates VoteAddToBlacklist record

### vote_remove
- **Purpose**: Vote to remove a validator from the blacklist
- **Parameters**:
  - `validator_identity_address`: The validator to remove from blacklist
  - `reason`: Explanation for the vote
- **Behavior**: Creates/updates Blacklist account and creates VoteRemoveFromBlacklist record

### unvote_add
- **Purpose**: Remove a previously cast vote to add a validator
- **Parameters**:
  - `validator_identity_address`: The validator for which to remove the vote
- **Behavior**: Closes the VoteAddToBlacklist account and decrements the tally

### unvote_remove
- **Purpose**: Remove a previously cast vote to remove a validator
- **Parameters**:
  - `validator_identity_address`: The validator for which to remove the vote
- **Behavior**: Closes the VoteRemoveFromBlacklist account and decrements the tally

### delegate
- **Purpose**: Delegate voting authority from a stake pool manager to another address
- **Parameters**: None (delegate address is specified as an account)
- **Behavior**: Creates a Delegation account allowing the delegate to vote on behalf of the manager
- **Requirements**: Must be signed by the stake pool manager

### undelegate
- **Purpose**: Remove a delegation and revoke the delegate's authority
- **Parameters**: None
- **Behavior**: Closes the Delegation account, revoking the delegate's authority
- **Requirements**: Must be signed by the original stake pool manager

## Development

### Prerequisites
- [Rust](https://rustup.rs/)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor Framework](https://www.anchor-lang.com/docs/installation)
- [Node.js](https://nodejs.org/)

### Building
```bash
anchor build
```

### Testing
```bash
# Use cargo test rather than anchor test as all unit tests use LiteSVM rather than the test validator local instance
cargo test
```

### Deploying
```bash
anchor deploy
```

## Usage Examples

### Query All Blacklisted Validators
Use `getProgramAccounts` to fetch all Blacklist accounts:

```typescript
const blacklists = await connection.getProgramAccounts(programId, {
  filters: [
    {
      memcmp: {
        offset: 0,
        bytes: bs58.encode(Buffer.from("blacklist")),
      },
    },
  ],
});
```

### Query Votes for a Specific Validator
```typescript
const votes = await connection.getProgramAccounts(programId, {
  filters: [
    {
      memcmp: {
        offset: 40,
        bytes: validatorIdentityAddress.toBase58(),
      },
    },
  ],
});
```

### Query All Delegations
```typescript
const delegations = await connection.getProgramAccounts(programId, {
  filters: [
    {
      memcmp: {
        offset: 0,
        bytes: bs58.encode(Buffer.from("delegation")),
      },
    },
  ],
});
```

### Vote with Delegation
When voting with delegated authority, include the delegation account:
```typescript
// The delegate signs the transaction, but the delegation account
// proves their authority to vote on behalf of the stake pool manager
await program.methods
  .voteAdd(validatorAddress, reason)
  .accounts({
    blacklist: blacklistPda,
    voteAdd: voteAddPda,
    stakePool: stakePoolAddress,
    authority: delegateKeypair.publicKey, // The delegate
    delegation: delegationPda, // Proves delegation authority
    systemProgram: SystemProgram.programId,
  })
  .signers([delegateKeypair])
  .rpc();
```

## Security Considerations

- Only authenticated stake pool operators can vote
- Each operator can only cast one vote per validator per direction (add/remove)
- All votes are permanently recorded on-chain for transparency
- Account seeds ensure deterministic addressing and prevent conflicts
- Delegation authority can only be created by the original stake pool manager
- Delegated authority can be revoked at any time by the original manager
- The program validates stake pool manager authority by deserializing the SPL Stake Pool state

## License

This project is licensed under the MIT License.
