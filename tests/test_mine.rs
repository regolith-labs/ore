use std::str::FromStr;

use ore::{
    instruction::{CreateProofArgs, MineArgs, OreInstruction},
    state::{Proof, Treasury},
    utils::AccountDeserialize,
    BUS, PROOF, TREASURY,
};
use solana_program::{
    clock::Clock,
    epoch_schedule::DEFAULT_SLOTS_PER_EPOCH,
    instruction::{AccountMeta, Instruction},
    keccak::{hashv, Hash as KeccakHash},
    pubkey::Pubkey,
    system_program, sysvar,
};
use solana_program_test::{processor, BanksClient, ProgramTest};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn test_mine() {
    // Setup
    let (mut banks, payer, hash) = setup_program_test_env().await;

    // Build proof ix
    let proof_pda = Pubkey::find_program_address(&[PROOF, payer.pubkey().as_ref()], &ore::id());
    let ix_0 = Instruction {
        program_id: ore::id(),
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(proof_pda.0, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: [
            OreInstruction::CreateProof.to_vec(),
            CreateProofArgs { bump: proof_pda.1 }.to_bytes().to_vec(),
        ]
        .concat(),
    };

    // Submit tx
    let tx = Transaction::new_signed_with_payer(&[ix_0], Some(&payer.pubkey()), &[&payer], hash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Assert proof state
    let proof_account = banks.get_account(proof_pda.0).await.unwrap().unwrap();
    assert_eq!(proof_account.owner, ore::id());
    let proof = Proof::try_from_bytes(&proof_account.data).unwrap();

    // Assert proof state
    let treasury_pda = Pubkey::find_program_address(&[TREASURY], &ore::id());
    let treasury_account = banks.get_account(treasury_pda.0).await.unwrap().unwrap();
    let treasury = Treasury::try_from_bytes(&treasury_account.data).unwrap();

    // Find next hash
    let (next_hash, nonce) = find_next_hash(
        proof.hash.into(),
        treasury.difficulty.into(),
        payer.pubkey(),
    );

    // Build mine ix
    let bus_pda = Pubkey::find_program_address(&[BUS, &[0]], &ore::id());
    let treasury_pda = Pubkey::find_program_address(&[TREASURY], &ore::id());
    let ix_1 = Instruction {
        program_id: ore::id(),
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(bus_pda.0, false),
            AccountMeta::new(proof_pda.0, false),
            AccountMeta::new_readonly(treasury_pda.0, false),
            // AccountMeta::new(treasury_pda.0, false),
            // AccountMeta::new(proof_pda.0, false),
            AccountMeta::new_readonly(sysvar::slot_hashes::id(), false),
        ],
        data: [
            OreInstruction::Mine.to_vec(),
            MineArgs {
                hash: next_hash.into(),
                nonce: nonce.to_le_bytes(),
            }
            .to_bytes()
            .to_vec(),
        ]
        .concat(),
    };

    // Submit tx
    let tx = Transaction::new_signed_with_payer(&[ix_1], Some(&payer.pubkey()), &[&payer], hash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // TODO Assert proof state
    // TODO Assert bus state
}

fn find_next_hash(hash: KeccakHash, difficulty: KeccakHash, signer: Pubkey) -> (KeccakHash, u64) {
    let mut next_hash: KeccakHash;
    let mut nonce = 0u64;
    loop {
        next_hash = hashv(&[
            hash.to_bytes().as_slice(),
            signer.to_bytes().as_slice(),
            nonce.to_be_bytes().as_slice(),
        ]);
        if next_hash.le(&difficulty) {
            break;
        } else {
            println!("Invalid hash: {} Nonce: {:?}", next_hash.to_string(), nonce);
        }
        nonce += 1;
    }
    (next_hash, nonce)
}

async fn setup_program_test_env() -> (BanksClient, Keypair, solana_program::hash::Hash) {
    let mut program_test = ProgramTest::new("ore", ore::ID, processor!(ore::process_instruction));
    program_test.prefer_bpf(true);

    // Busses
    program_test.add_account_with_base64_data(
        Pubkey::from_str("2uwqyH2gKqstgAFCSniirx73X4iQek5ETc2vVJKUiNMg").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD/AAAAAAAAAICy5g4AAAAA",
    );
    program_test.add_account_with_base64_data(
        Pubkey::from_str("FRMC6jVczm1cRaEs5EhDsfw7X8vsmSDpf3bJWVkawngu").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD+AAAAAQAAAICy5g4AAAAA",
    );
    program_test.add_account_with_base64_data(
        Pubkey::from_str("9nWyycs4GHjnLujPR2sbA1A8K8CkiLc5VzxWUD4hg2uM").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD/AAAAAgAAAICy5g4AAAAA",
    );
    program_test.add_account_with_base64_data(
        Pubkey::from_str("Kt7kqD3MyvxLbj4ek9urXUxkDoxaMuQn82K2VdYD1jM").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD6AAAAAwAAAICy5g4AAAAA",
    );
    program_test.add_account_with_base64_data(
        Pubkey::from_str("8r9mXYnFQXhwrNfvatGUTxbbNSqxScuCwp4sBTSxDVTJ").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD9AAAABAAAAICy5g4AAAAA",
    );
    program_test.add_account_with_base64_data(
        Pubkey::from_str("D9cEH32k8p9uWc4w5RrStK9rWssU8NuX1Dg5YaUim4wL").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD/AAAABQAAAICy5g4AAAAA",
    );
    program_test.add_account_with_base64_data(
        Pubkey::from_str("H1RKMYADPzd4C1j1RZu51NvRSVktoTYEJyeVy98Kmdyu").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD/AAAABgAAAICy5g4AAAAA",
    );
    program_test.add_account_with_base64_data(
        Pubkey::from_str("3XbdZNbBjjp8qnDJjv1RxaKisyfx6ahznYkSigs6dayy").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD5AAAABwAAAICy5g4AAAAA",
    );

    // Treasury (difficulty = MAX)
    program_test.add_account_with_base64_data(
        Pubkey::from_str("67PLJej6iZm915WbEu6NLeZtRZtnHc5nSVQvkHRZyPiC").unwrap(),
        1559040,
        ore::id(),
        "ZgAAAAAAAAD/AAAAAAAAAI9MXkItHZzhz/U8d4MsXPzDSQZSRgZsJnNpvgvcborr//////////////////////////////////////////9kAAAAAAAAAPQBAAAAAAAAAAAAAAAAAAA=",
    );

    // Mint
    program_test.add_account_with_base64_data(
        Pubkey::from_str("DY4JVebraRXg9BGt4MRU4mvqHGDzmi2Ay1HGjDU5YeNf").unwrap(),
        1461600,
        spl_token::id(),
        "AQAAAEvtK9pjA/sPMEl3rhUgX8iz4/q0A5icrVGp0GdL3satAJQ1dwAAAAAJAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
    );

    // Treasury tokens
    program_test.add_account_with_base64_data(
        Pubkey::from_str("EH4tskvkeNqX5ce3FBr4oJob3FKSns9th7NvP28ZHsNL").unwrap(),
        2039280,
        spl_token::id(),
        "ukD7Oc0QjzbigRIB1x9/XLzAT3w7X0UTZ1NVeB85lRRL7SvaYwP7DzBJd64VIF/Is+P6tAOYnK1RqdBnS97GrQCUNXcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
    );

    // Set sysvar
    program_test.add_sysvar_account(
        sysvar::clock::id(),
        &Clock {
            slot: 10,
            epoch_start_timestamp: 0,
            epoch: 0,
            leader_schedule_epoch: DEFAULT_SLOTS_PER_EPOCH,
            unix_timestamp: 100,
        },
    );

    program_test.start().await
}
