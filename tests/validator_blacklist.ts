import {BN, Program, web3} from "@coral-xyz/anchor";
import anchor from "@coral-xyz/anchor";
import fs from 'fs'
import {
    PublicKey,
    Keypair,
    SystemProgram,
    LAMPORTS_PER_SOL,
    Transaction,
    TransactionInstruction,
    AccountInfo
} from "@solana/web3.js";
import {FailedTransactionMetadata, LiteSVM} from "litesvm";
import {assert, expect} from "chai";
import * as fs from "fs";
import {struct, u8, publicKey, u64} from '@coral-xyz/borsh';
import {ValidatorBlacklist} from "../target/types/validator_blacklist";
import {InstructionErrorCustom, TransactionErrorInstructionError, TransactionMetadata} from "litesvm/dist/internal";
import {STAKE_POOL_PROGRAM_ID} from "@solana/spl-stake-pool";

/**
 * Copied from @solana/spl-stake-pool for compatibility.
 * We only need the account data structure, not the full package.
 */
type StakePoolLayout = {
    accountType: number;
    manager: PublicKey;
    staker: PublicKey;
    stakeDepositAuthority: PublicKey;
    stakeWithdrawBumpSeed: number;
    validatorList: PublicKey;
    reserveStake: PublicKey;
    poolMint: PublicKey;
    managerFeeAccount: PublicKey;
    tokenProgramId: PublicKey;
    totalLamports: BN;
    poolTokenSupply: BN;
    lastUpdateEpoch: BN;
};

const StakePoolLayout = struct<StakePoolLayout>([
    u8('accountType'),
    publicKey('manager'),
    publicKey('staker'),
    publicKey('stakeDepositAuthority'),
    u8('stakeWithdrawBumpSeed'),
    publicKey('validatorList'),
    publicKey('reserveStake'),
    publicKey('poolMint'),
    publicKey('managerFeeAccount'),
    publicKey('tokenProgramId'),
    u64('totalLamports'),
    u64('poolTokenSupply'),
    u64('lastUpdateEpoch'),
]);

function expectInstructionErrorCustomCode(result: FailedTransactionMetadata, code: number) {

    const error = result.err();

    expect(error).to.be.instanceOf(TransactionErrorInstructionError);

    const instructionError = (error as TransactionErrorInstructionError).err();

    expect(instructionError).to.be.instanceOf(InstructionErrorCustom);

    const errorCode = (instructionError as InstructionErrorCustom).code;

    if (errorCode != code) {
        console.log("Expected error code:", code, "but got:", errorCode);
        console.log("Full error:", result.toString());
    }
    expect(errorCode).to.equal(code);
}

function expectSuccessfulTransaction(result: TransactionMetadata | FailedTransactionMetadata) {

    expect(result).to.be.instanceOf(TransactionMetadata,
        `Expected successful transaction metadata, got ${result.toString()}`);
}

async function cloneAccount(path: string, svm: LiteSVM, account: PublicKey, modifier = (data: Buffer) => data, owner = STAKE_POOL_PROGRAM_ID) {
    const {account: accountInfo} = JSON.parse(fs.readFileSync(path).toString());
    if (accountInfo) {
        svm.setAccount(account, {
            lamports: accountInfo.lamports,
            data: modifier(Buffer.from(accountInfo.data[0], "base64")),
            owner,
            executable: accountInfo.executable,
        });
    } else {
        throw new Error(`Failed to clone account ${account.toString()}`);
    }
};

describe("Validator Blacklist with LiteSVM", () => {
    let svm: LiteSVM;
    let programId: PublicKey;
    let program: Program<ValidatorBlacklist>;

    // Stake pool account (vSOL stake pool)
    const stakePoolAddress1 = Keypair.generate().publicKey;
    const stakePoolAddress2 = Keypair.generate().publicKey;
    const stakePoolBadProgram = Keypair.generate().publicKey;
    const stakePoolManager = Keypair.generate();

    let delegateAuthority: Keypair;
    let unauthorizedUser: Keypair;
    let validatorToBlacklist: PublicKey;
    let configAdmin: Keypair;
    let configAddress: Keypair;

    // PDAs
    let delegationPda: PublicKey;
    let blacklistPda: PublicKey;
    let voteAddPda: PublicKey;
    let voteRemovePda: PublicKey;

    before(async () => {
        // Initialize LiteSVM
        svm = new LiteSVM();

        // Generate test accounts
        delegateAuthority = Keypair.generate();
        unauthorizedUser = Keypair.generate();
        validatorToBlacklist = Keypair.generate().publicKey;
        configAdmin = Keypair.generate();

        // Get some SOL
        svm.airdrop(stakePoolManager.publicKey, BigInt(10 * LAMPORTS_PER_SOL));
        svm.airdrop(delegateAuthority.publicKey, BigInt(10 * LAMPORTS_PER_SOL));
        svm.airdrop(unauthorizedUser.publicKey, BigInt(10 * LAMPORTS_PER_SOL));
        svm.airdrop(configAdmin.publicKey, BigInt(10 * LAMPORTS_PER_SOL));

        // Clone vSOL stake pool state account and set the manager to our mocked manager key for testing
        await cloneAccount("./tests/accounts/stakePool.json", svm, stakePoolAddress1, (data) => {
            const stakePoolDeserialized = StakePoolLayout.decode(data);
            stakePoolDeserialized.manager = stakePoolManager.publicKey;
            const buffer = Buffer.alloc(data.length);

            StakePoolLayout.encode(stakePoolDeserialized, buffer);

            return buffer;
        });

        await cloneAccount("./tests/accounts/stakePool.json", svm, stakePoolAddress2, (data) => {
            const stakePoolDeserialized = StakePoolLayout.decode(data);
            stakePoolDeserialized.manager = stakePoolManager.publicKey;
            const buffer = Buffer.alloc(data.length);

            StakePoolLayout.encode(stakePoolDeserialized, buffer);

            return buffer;
        });

        await cloneAccount("./tests/accounts/stakePool.json", svm, stakePoolAddress2, (data) => {
            const stakePoolDeserialized = StakePoolLayout.decode(data);
            stakePoolDeserialized.manager = stakePoolManager.publicKey;
            const buffer = Buffer.alloc(data.length);

            StakePoolLayout.encode(stakePoolDeserialized, buffer);

            return buffer;
        });

        await cloneAccount("./tests/accounts/stakePool.json", svm, stakePoolBadProgram, (data) => {
            const stakePoolDeserialized = StakePoolLayout.decode(data);
            stakePoolDeserialized.manager = stakePoolManager.publicKey;
            const buffer = Buffer.alloc(data.length);

            StakePoolLayout.encode(stakePoolDeserialized, buffer);

            return buffer;
        }, SystemProgram.programId);

        expect(svm.getAccount(stakePoolAddress1)).to.be.not.null;
        expect(svm.getAccount(stakePoolAddress1)?.data.length).to.be.equal(611);
        expect(StakePoolLayout.decode(Buffer.from(svm.getAccount(stakePoolAddress1)?.data)).manager.toString()).to.be.equal(stakePoolManager.publicKey.toString());

        // Load and deploy the program
        const programKeypairData = JSON.parse(fs.readFileSync("./target/deploy/validator_blacklist-keypair.json", "utf8"));
        const programKeypair = Keypair.fromSecretKey(new Uint8Array(programKeypairData));

        programId = programKeypair.publicKey;
        svm.addProgramFromFile(programId, "./target/deploy/validator_blacklist.so");

        // Load program IDL and create Anchor program instance
        const idl = JSON.parse(fs.readFileSync("./target/idl/validator_blacklist.json", "utf8"));
        // Create a minimal provider-like object for program creation
        const mockProvider = {
            connection: {
                getAccountInfoAndContext: async (address: PublicKey, commitmentOrConfig) => {

                    const account = svm.getAccount(address);

                    if (account === null) {
                        return null;
                    }

                    return {
                        value: {
                            data: Buffer.from(account.data),
                            executable: account.executable,
                            owner: account.owner,
                            lamports: account.lamports,
                            rentEpoch: account.rentEpoch
                        },
                        context: undefined,
                    };
                },
            } as any as web3.Connection,
            publicKey: programId,
            signTransaction: () => Promise.resolve(),
            signAllTransactions: () => Promise.resolve([])
        };

        program = new Program(idl, mockProvider) as Program<ValidatorBlacklist>;

        // Calculate config PDA
        configAddress = Keypair.generate();

        // Initialize the config first
        const initConfigIx = await program.methods
            .initConfig(
                new BN(1000000000), // 1 SOL minimum TVL
                [STAKE_POOL_PROGRAM_ID] // Allow the vSOL stake pool program
            )
            .accounts({
                config: configAddress.publicKey,
                admin: configAdmin.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .instruction();

        const initConfigTx = new Transaction().add(initConfigIx);
        initConfigTx.feePayer = configAdmin.publicKey;
        initConfigTx.recentBlockhash = svm.latestBlockhash();
        initConfigTx.sign(configAdmin, configAddress);
        const initConfigResult = svm.sendTransaction(initConfigTx);
        expectSuccessfulTransaction(initConfigResult);

        // Calculate PDAs with config included
        [delegationPda] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("delegation"),
                configAddress.publicKey.toBuffer(),
                stakePoolAddress1.toBuffer()
            ],
            programId
        );

        [blacklistPda] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("blacklist"),
                configAddress.publicKey.toBuffer(),
                validatorToBlacklist.toBuffer()
            ],
            programId
        );

        [voteAddPda] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vote_add"),
                configAddress.publicKey.toBuffer(),
                stakePoolAddress1.toBuffer(),
                validatorToBlacklist.toBuffer()
            ],
            programId
        );

        [voteRemovePda] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vote_remove"),
                configAddress.publicKey.toBuffer(),
                stakePoolAddress1.toBuffer(),
                validatorToBlacklist.toBuffer()
            ],
            programId
        );
    });

    describe("Config Management", () => {
        it("Should have initialized config correctly", async () => {
            const configAccount = await program.account.config.fetch(configAddress.publicKey);
            expect(configAccount.admin.toString()).to.equal(configAdmin.publicKey.toString());
            expect(configAccount.minTvl.toString()).to.equal("1000000000");
            expect(configAccount.allowedPrograms).to.have.length(1);
            expect(configAccount.allowedPrograms[0].toString()).to.equal(STAKE_POOL_PROGRAM_ID.toString());
        });

        it("Should allow admin to update config", async () => {
            const updateConfigIx = await program.methods
                .updateConfig(
                    new BN(2000000000), // 2 SOL minimum TVL
                    null // Don't update allowed programs
                )
                .accounts({
                    config: configAddress.publicKey,
                    admin: configAdmin.publicKey,
                })
                .instruction();

            const tx = new Transaction().add(updateConfigIx);
            tx.feePayer = configAdmin.publicKey;
            tx.recentBlockhash = svm.latestBlockhash();
            tx.sign(configAdmin);

            const result = svm.sendTransaction(tx);
            expectSuccessfulTransaction(result);

            const configAccount = await program.account.config.fetch(configAddress.publicKey);
            expect(configAccount.minTvl.toString()).to.equal("2000000000");
        });

        it("Should allow admin to update admin", async () => {
            const newAdmin = Keypair.generate();
            svm.airdrop(newAdmin.publicKey, BigInt(LAMPORTS_PER_SOL));

            const updateAdminIx = await program.methods
                .updateConfigAdmin(newAdmin.publicKey)
                .accounts({
                    config: configAddress.publicKey,
                    admin: configAdmin.publicKey,
                })
                .instruction();

            const tx = new Transaction().add(updateAdminIx);
            tx.feePayer = configAdmin.publicKey;
            tx.recentBlockhash = svm.latestBlockhash();
            tx.sign(configAdmin);

            const result = svm.sendTransaction(tx);
            expectSuccessfulTransaction(result);

            const configAccount = await program.account.config.fetch(configAddress.publicKey);
            expect(configAccount.admin.toString()).to.equal(newAdmin.publicKey.toString());

            // Update configAdmin for subsequent tests
            configAdmin = newAdmin;
        });
    });

    describe("Stake Pool Validation", () => {
        it("Should validate stake pool with mock manager", async () => {
            const stakePoolAccount = svm.getAccount(stakePoolAddress1);
            expect(stakePoolAccount).to.not.be.null;

            const stakePool = StakePoolLayout.decode(Buffer.from(stakePoolAccount.data));

            expect(stakePool.manager.toString()).to.equal(stakePoolManager.publicKey.toString());
        });
    });

    describe("PDA Calculation Tests", () => {
        it("Should calculate consistent PDAs for delegation with config", async () => {
            const [calculatedPda, bump] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("delegation"),
                    configAddress.publicKey.toBuffer(),
                    stakePoolAddress1.toBuffer()
                ],
                programId
            );

            expect(calculatedPda.toString()).to.equal(delegationPda.toString());
            expect(bump).to.be.a('number');
            expect(bump).to.be.lessThan(256);

            console.log("Delegation PDA:", delegationPda.toString());
            console.log("Delegation bump:", bump);
        });

        it("Should calculate consistent PDAs for blacklist with config", async () => {
            const [calculatedPda, bump] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("blacklist"),
                    configAddress.publicKey.toBuffer(),
                    validatorToBlacklist.toBuffer()
                ],
                programId
            );

            expect(calculatedPda.toString()).to.equal(blacklistPda.toString());
            expect(bump).to.be.a('number');

            console.log("Blacklist PDA:", blacklistPda.toString());
            console.log("Blacklist bump:", bump);
        });

        it("Should calculate different PDAs for different validators", async () => {
            const validator2 = Keypair.generate().publicKey;

            const [blacklist1Pda] = PublicKey.findProgramAddressSync(
                [Buffer.from("blacklist"), configAddress.publicKey.toBuffer(), validatorToBlacklist.toBuffer()],
                programId
            );

            const [blacklist2Pda] = PublicKey.findProgramAddressSync(
                [Buffer.from("blacklist"), configAddress.publicKey.toBuffer(), validator2.toBuffer()],
                programId
            );

            expect(blacklist1Pda.toString()).to.not.equal(blacklist2Pda.toString());
        });
    });

    describe("Instruction Execution Tests", () => {
        describe("Delegate Instruction", () => {
            it("Should successfully create a delegation", async () => {

                const delegateIx = await program.methods
                    .delegate()
                    .accountsPartial({
                        config: configAddress.publicKey,
                        stakePool: stakePoolAddress1,
                        delegation: delegationPda,
                        manager: stakePoolManager.publicKey,
                        delegate: delegateAuthority.publicKey,
                        systemProgram: SystemProgram.programId,
                    })
                    .instruction();

                const tx = new Transaction().add(delegateIx);
                tx.feePayer = stakePoolManager.publicKey;
                tx.recentBlockhash = svm.latestBlockhash();
                tx.sign(stakePoolManager);

                const result = svm.sendTransaction(tx);
                expectSuccessfulTransaction(result);

                // Verify delegation account was created
                const delegationAccount = await program.account.delegation.fetchNullable(delegationPda);
                expect(delegationAccount).to.not.be.null;
                expect(delegationAccount.stakePool.toString()).to.equal(stakePoolAddress1.toString());
                expect(delegationAccount.manager.toString()).to.equal(stakePoolManager.publicKey.toString());
                expect(delegationAccount.delegate.toString()).to.equal(delegateAuthority.publicKey.toString());

            });

            it("Should fail to create delegation with wrong manager", async () => {
                const wrongManager = Keypair.generate();
                svm.airdrop(wrongManager.publicKey, BigInt(LAMPORTS_PER_SOL));

                const [wrongDelegationPda] = PublicKey.findProgramAddressSync(
                    [
                        Buffer.from("delegation"),
                        configAddress.publicKey.toBuffer(),
                        stakePoolAddress2.toBuffer()
                    ],
                    programId
                );

                const delegateIx = await program.methods
                    .delegate()
                    .accountsPartial({
                        config: configAddress.publicKey,
                        stakePool: stakePoolAddress2,
                        delegation: wrongDelegationPda,
                        manager: wrongManager.publicKey,
                        delegate: delegateAuthority.publicKey,
                        systemProgram: SystemProgram.programId,
                    })
                    .instruction();

                const tx = new Transaction().add(delegateIx);
                tx.feePayer = wrongManager.publicKey;
                tx.recentBlockhash = svm.latestBlockhash();
                tx.sign(wrongManager);

                const result = svm.sendTransaction(tx);

                expect(result).to.be.instanceOf(FailedTransactionMetadata);
                expectInstructionErrorCustomCode(result as FailedTransactionMetadata, 6000);
            });
        });

        describe("Vote Add Instruction", () => {
            it("Should fail to vote with unauthorized user", async () => {
                const [unauthorizedVoteAddPda] = PublicKey.findProgramAddressSync(
                    [
                        Buffer.from("vote_add"),
                        configAddress.publicKey.toBuffer(),
                        stakePoolAddress2.toBuffer(),
                        validatorToBlacklist.toBuffer()
                    ],
                    programId
                );

                const voteAddIx = await program.methods
                    .voteAdd(validatorToBlacklist, "Unauthorized vote")
                    .accountsPartial({
                        config: configAddress.publicKey,
                        stakePool: stakePoolAddress2,
                        blacklist: blacklistPda,
                        voteAdd: unauthorizedVoteAddPda,
                        delegation: null,
                        authority: unauthorizedUser.publicKey,
                        systemProgram: SystemProgram.programId,
                    })
                    .instruction();

                const tx = new Transaction().add(voteAddIx);
                tx.feePayer = unauthorizedUser.publicKey;
                tx.recentBlockhash = svm.latestBlockhash();
                tx.sign(unauthorizedUser);

                const result = svm.sendTransaction(tx);

                expect(result).to.be.instanceOf(FailedTransactionMetadata);
                expectInstructionErrorCustomCode(result as FailedTransactionMetadata, 6000);

            });
            it("Should successfully vote to add validator to blacklist", async () => {
                const reason = "Malicious behavior detected";

                const voteAddIx = await program.methods
                    .voteAdd(validatorToBlacklist, reason)
                    .accountsPartial({
                        config: configAddress.publicKey,
                        stakePool: stakePoolAddress1,
                        blacklist: blacklistPda,
                        voteAdd: voteAddPda,
                        delegation: null,
                        authority: stakePoolManager.publicKey,
                        systemProgram: SystemProgram.programId,
                    })
                    .instruction();

                const tx = new Transaction().add(voteAddIx);
                tx.feePayer = stakePoolManager.publicKey;
                tx.recentBlockhash = svm.latestBlockhash();
                tx.sign(stakePoolManager);

                const result = svm.sendTransaction(tx);

                expectSuccessfulTransaction(result);

            });

        });

        describe("Vote Remove Instruction", () => {
            it("Should successfully vote to remove validator from blacklist", async () => {
                const reason = "False positive, validator is legitimate";

                const voteRemoveIx = await program.methods
                    .voteRemove(validatorToBlacklist, reason)
                    .accountsPartial({
                        config: configAddress.publicKey,
                        stakePool: stakePoolAddress1,
                        blacklist: blacklistPda,
                        voteRemove: voteRemovePda,
                        delegation: null,
                        authority: stakePoolManager.publicKey,
                        systemProgram: SystemProgram.programId,
                    })
                    .instruction();

                const tx = new Transaction().add(voteRemoveIx);
                tx.feePayer = stakePoolManager.publicKey;
                tx.recentBlockhash = svm.latestBlockhash();
                tx.sign(stakePoolManager);

                const result = svm.sendTransaction(tx);
                expectSuccessfulTransaction(result);
            });
        });

        describe("Unvote Add Instruction", () => {
            it("Should successfully remove a previous add vote", async () => {
                const unvoteAddIx = await program.methods
                    .unvoteAdd(validatorToBlacklist)
                    .accountsPartial({
                        config: configAddress.publicKey,
                        stakePool: stakePoolAddress1,
                        blacklist: blacklistPda,
                        voteAdd: voteAddPda,
                        delegation: null,
                        authority: stakePoolManager.publicKey,
                    })
                    .instruction();

                const tx = new Transaction().add(unvoteAddIx);
                tx.feePayer = stakePoolManager.publicKey;
                tx.recentBlockhash = svm.latestBlockhash();
                tx.sign(stakePoolManager);

                const result = svm.sendTransaction(tx);
                expectSuccessfulTransaction(result);


                // Verify vote PDA was closed
                const voteAddAccount = svm.getAccount(voteAddPda);
                expect(voteAddAccount.lamports).to.equal(0);
                expect(voteAddAccount.owner.toBase58()).to.eq(SystemProgram.programId.toBase58());

            });
        });

        describe("Delegated Authority Tests", () => {
            it("Should fail delegated vote with invalid delegation", async () => {
                const wrongDelegate = Keypair.generate();
                svm.airdrop(wrongDelegate.publicKey, BigInt(LAMPORTS_PER_SOL));

                const [wrongDelegatedVoteAddPda] = PublicKey.findProgramAddressSync(
                    [
                        Buffer.from("vote_add"),
                        configAddress.publicKey.toBuffer(),
                        stakePoolAddress1.toBuffer(),
                        validatorToBlacklist.toBuffer()
                    ],
                    programId
                );

                const voteAddIx = await program.methods
                    .voteAdd(validatorToBlacklist, "Wrong delegate")
                    .accountsPartial({
                        config: configAddress.publicKey,
                        stakePool: stakePoolAddress1,
                        blacklist: blacklistPda,
                        voteAdd: wrongDelegatedVoteAddPda,
                        delegation: delegationPda,
                        authority: wrongDelegate.publicKey,
                        systemProgram: SystemProgram.programId,
                    })
                    .instruction();

                const tx = new Transaction().add(voteAddIx);
                tx.feePayer = wrongDelegate.publicKey;
                tx.recentBlockhash = svm.latestBlockhash();
                tx.sign(wrongDelegate);

                const result = svm.sendTransaction(tx);

                expect(result).to.be.instanceOf(FailedTransactionMetadata);
                expectInstructionErrorCustomCode(result as FailedTransactionMetadata, 6002);
            });
            it("Should successfully vote with delegated authority", async () => {

                // Calculate PDA for delegated authority vote
                const [delegatedVoteAddPda] = PublicKey.findProgramAddressSync(
                    [
                        Buffer.from("vote_add"),
                        configAddress.publicKey.toBuffer(),
                        stakePoolAddress1.toBuffer(),
                        validatorToBlacklist.toBuffer()
                    ],
                    programId
                );

                const reason = "some valid reason for blacklisting a validator";
                const voteAddIx = await program.methods
                    .voteAdd(validatorToBlacklist, reason)
                    .accountsPartial({
                        config: configAddress.publicKey,
                        stakePool: stakePoolAddress1,
                        blacklist: blacklistPda,
                        voteAdd: delegatedVoteAddPda,
                        delegation: delegationPda,
                        authority: delegateAuthority.publicKey,
                        systemProgram: SystemProgram.programId,
                    })
                    .instruction();

                const tx = new Transaction().add(voteAddIx);
                tx.feePayer = delegateAuthority.publicKey;
                tx.recentBlockhash = svm.latestBlockhash();
                tx.sign(delegateAuthority);

                const result = svm.sendTransaction(tx);
                expectSuccessfulTransaction(result);

                const voteAddAccount = await program.account.voteAddToBlacklist.fetchNullable(delegatedVoteAddPda);
                expect(voteAddAccount).to.not.be.null;
                expect(voteAddAccount.reason).to.equal(reason);
                expect(voteAddAccount.stakePool.toString()).to.equal(stakePoolAddress1.toString());
            });

        });


        describe("Undelegate Instruction", () => {
            it("Should successfully remove delegation", async () => {
                const undelegateIx = await program.methods
                    .undelegate()
                    .accounts({
                        config: configAddress.publicKey,
                        stakePool: stakePoolAddress1,
                        delegation: delegationPda,
                        manager: stakePoolManager.publicKey,
                    })
                    .instruction();

                const tx = new Transaction().add(undelegateIx);
                tx.feePayer = stakePoolManager.publicKey;
                tx.recentBlockhash = svm.latestBlockhash();
                tx.sign(stakePoolManager);

                const result = svm.sendTransaction(tx);
                expectSuccessfulTransaction(result);

                console.log("Undelegate instruction properly structured");
            });
        });

        describe("Edge Cases and Error Conditions", () => {
            it("Should handle voting on multiple validators", async () => {
                const validator2 = Keypair.generate().publicKey;
                const [blacklist2Pda] = PublicKey.findProgramAddressSync(
                    [Buffer.from("blacklist"), configAddress.publicKey.toBuffer(), validator2.toBuffer()],
                    programId
                );

                const [vote2AddPda] = PublicKey.findProgramAddressSync(
                    [
                        Buffer.from("vote_add"),
                        configAddress.publicKey.toBuffer(),
                        stakePoolAddress1.toBuffer(),
                        validator2.toBuffer()
                    ],
                    programId
                );

                const voteAddIx = await program.methods
                    .voteAdd(validator2, "Second validator")
                    .accountsPartial({
                        config: configAddress.publicKey,
                        stakePool: stakePoolAddress1,
                        blacklist: blacklist2Pda,
                        voteAdd: vote2AddPda,
                        delegation: null,
                        authority: stakePoolManager.publicKey,
                        systemProgram: SystemProgram.programId,
                    })
                    .instruction();

                const tx = new Transaction().add(voteAddIx);
                tx.feePayer = stakePoolManager.publicKey;
                tx.recentBlockhash = svm.latestBlockhash();
                tx.sign(stakePoolManager);

                const result = svm.sendTransaction(tx);
                expectSuccessfulTransaction(result);
            });

            it("Should handle transaction with invalid PDA", async () => {
                const invalidPda = Keypair.generate().publicKey;

                const voteAddIx = await program.methods
                    .voteAdd(validatorToBlacklist, "Invalid PDA test")
                    .accountsPartial({
                        config: configAddress.publicKey,
                        stakePool: stakePoolAddress1,
                        blacklist: invalidPda, // Invalid PDA
                        voteAdd: voteAddPda,
                        delegation: null,
                        authority: stakePoolManager.publicKey,
                        systemProgram: SystemProgram.programId,
                    })
                    .instruction();

                const tx = new Transaction().add(voteAddIx);
                tx.feePayer = stakePoolManager.publicKey;
                tx.recentBlockhash = svm.latestBlockhash();
                tx.sign(stakePoolManager);

                const result = svm.sendTransaction(tx);
                expect(result).to.be.instanceOf(FailedTransactionMetadata);
                expectInstructionErrorCustomCode(result as FailedTransactionMetadata, 2006 /* seed constraint violation */);

            });

            it("Should fail with unauthorized stake pool program", async () => {
                // Update config to remove the stake pool from allowed programs
                const updateConfigIx = await program.methods
                    .updateConfig(
                        null,
                        [] // Empty allowed programs list
                    )
                    .accounts({
                        config: configAddress.publicKey,
                        admin: configAdmin.publicKey,
                    })
                    .instruction();

                const updateTx = new Transaction().add(updateConfigIx);
                updateTx.feePayer = configAdmin.publicKey;
                updateTx.recentBlockhash = svm.latestBlockhash();
                updateTx.sign(configAdmin);

                const updateResult = svm.sendTransaction(updateTx);
                expectSuccessfulTransaction(updateResult);

                // Try to vote with unauthorized program
                const [unauthorizedProgramVoteAddPda] = PublicKey.findProgramAddressSync(
                    [
                        Buffer.from("vote_add"),
                        configAddress.publicKey.toBuffer(),
                        stakePoolBadProgram.toBuffer(),
                        validatorToBlacklist.toBuffer()
                    ],
                    programId
                );

                const voteAddIx = await program.methods
                    .voteAdd(validatorToBlacklist, "Should fail due to unauthorized program")
                    .accountsPartial({
                        config: configAddress.publicKey,
                        stakePool: stakePoolBadProgram,
                        blacklist: blacklistPda,
                        voteAdd: unauthorizedProgramVoteAddPda,
                        delegation: null,
                        authority: stakePoolManager.publicKey,
                        systemProgram: SystemProgram.programId,
                    })
                    .instruction();

                const voteTx = new Transaction().add(voteAddIx);
                voteTx.feePayer = stakePoolManager.publicKey;
                voteTx.recentBlockhash = svm.latestBlockhash();
                voteTx.sign(stakePoolManager);

                const voteResult = svm.sendTransaction(voteTx);
                expect(voteResult).to.be.instanceOf(FailedTransactionMetadata);
                expectInstructionErrorCustomCode(voteResult as FailedTransactionMetadata, 6008); // UnauthorizedStakePoolProgram error code
            });

            it("Should fail with insufficient TVL", async () => {
                // Update config to require higher TVL than the stake pool has
                const currentStakePool = StakePoolLayout.decode(Buffer.from(svm.getAccount(stakePoolAddress2)?.data));
                const higherTvl = new BN(currentStakePool.totalLamports.toString()).add(new BN(1000000000));

                const updateConfigIx = await program.methods
                    .updateConfig(
                        higherTvl,
                        null
                    )
                    .accounts({
                        config: configAddress.publicKey,
                        admin: configAdmin.publicKey,
                    })
                    .instruction();

                const updateTx = new Transaction().add(updateConfigIx);
                updateTx.feePayer = configAdmin.publicKey;
                updateTx.recentBlockhash = svm.latestBlockhash();
                updateTx.sign(configAdmin);

                const updateResult = svm.sendTransaction(updateTx);
                expectSuccessfulTransaction(updateResult);

                // Try to vote with insufficient TVL
                const [insufficientTvlVoteAddPda] = PublicKey.findProgramAddressSync(
                    [
                        Buffer.from("vote_add"),
                        configAddress.publicKey.toBuffer(),
                        stakePoolAddress2.toBuffer(),
                        validatorToBlacklist.toBuffer()
                    ],
                    programId
                );

                const voteAddIx = await program.methods
                    .voteAdd(validatorToBlacklist, "Should fail due to TVL")
                    .accountsPartial({
                        config: configAddress.publicKey,
                        stakePool: stakePoolAddress2,
                        blacklist: blacklistPda,
                        voteAdd: insufficientTvlVoteAddPda,
                        delegation: null,
                        authority: stakePoolManager.publicKey,
                        systemProgram: SystemProgram.programId,
                    })
                    .instruction();

                const voteTx = new Transaction().add(voteAddIx);
                voteTx.feePayer = stakePoolManager.publicKey;
                voteTx.recentBlockhash = svm.latestBlockhash();
                voteTx.sign(stakePoolManager);

                const voteResult = svm.sendTransaction(voteTx);
                expect(voteResult).to.be.instanceOf(FailedTransactionMetadata);
                expectInstructionErrorCustomCode(voteResult as FailedTransactionMetadata, 6007); // InsufficientTvl error code
            });

        });
    });
});
