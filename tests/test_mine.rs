use std::{mem::size_of, str::FromStr};

use ore::{
    instruction::{MineArgs, OreInstruction},
    state::{Bus, Proof, Treasury},
    utils::{AccountDeserialize, Discriminator},
    BUS_ADDRESSES, BUS_COUNT, INITIAL_REWARD_RATE, MINT_ADDRESS, PROOF, START_AT, TOKEN_DECIMALS,
    TREASURY, TREASURY_ADDRESS,
};
use rand::{distributions::Uniform, Rng};
use solana_program::{
    clock::Clock,
    epoch_schedule::DEFAULT_SLOTS_PER_EPOCH,
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    keccak::{hashv, Hash as KeccakHash},
    native_token::LAMPORTS_PER_SOL,
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    slot_hashes::SlotHash,
    system_program, sysvar,
};
use solana_program_test::{processor, BanksClient, ProgramTest};
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::state::{AccountState, Mint};

#[tokio::test]
async fn test_mine() {
    // Setup
    let (mut banks, payer, _, blockhash) = setup_program_test_env(true).await;

    // Submit register tx
    let proof_pda = Pubkey::find_program_address(&[PROOF, payer.pubkey().as_ref()], &ore::id());
    let ix = ore::instruction::register(payer.pubkey());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Assert proof state
    let proof_account = banks.get_account(proof_pda.0).await.unwrap().unwrap();
    assert_eq!(proof_account.owner, ore::id());
    let proof = Proof::try_from_bytes(&proof_account.data).unwrap();
    assert_eq!(proof.authority, payer.pubkey());
    assert_eq!(proof.claimable_rewards, 0);
    assert_eq!(proof.hash, hashv(&[payer.pubkey().as_ref()]).into());
    assert_eq!(proof.total_hashes, 0);
    assert_eq!(proof.total_rewards, 0);

    // Find next hash
    let (next_hash, nonce) = find_next_hash(
        proof.hash.into(),
        KeccakHash::new_from_array([u8::MAX; 32]),
        payer.pubkey(),
    );

    // Submit mine tx
    let ix = ore::instruction::mine(payer.pubkey(), BUS_ADDRESSES[0], next_hash.into(), nonce);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Assert proof state
    let slot_hashes_account = banks
        .get_account(sysvar::slot_hashes::id())
        .await
        .unwrap()
        .unwrap();
    let slot_hash_bytes = &slot_hashes_account.data[0..size_of::<SlotHash>()];
    let proof_account = banks.get_account(proof_pda.0).await.unwrap().unwrap();
    assert_eq!(proof_account.owner, ore::id());
    let proof = Proof::try_from_bytes(&proof_account.data).unwrap();
    assert_eq!(proof.authority, payer.pubkey());
    assert_eq!(proof.claimable_rewards, INITIAL_REWARD_RATE);
    assert_eq!(
        proof.hash,
        hashv(&[&next_hash.as_ref(), slot_hash_bytes,]).into()
    );
    assert_eq!(proof.total_hashes, 1);
    assert_eq!(proof.total_rewards, INITIAL_REWARD_RATE);

    // Submit claim tx
    let amount = proof.claimable_rewards;
    let beneficiary_address = get_associated_token_address(&payer.pubkey(), &ore::MINT_ADDRESS);
    let token_ix = create_associated_token_account(
        &payer.pubkey(),
        &payer.pubkey(),
        &ore::MINT_ADDRESS,
        &spl_token::id(),
    );
    let ix = ore::instruction::claim(payer.pubkey(), beneficiary_address, amount);
    let tx = Transaction::new_signed_with_payer(
        &[token_ix, ix],
        Some(&payer.pubkey()),
        &[&payer],
        blockhash,
    );
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Assert proof state
    let proof_account = banks.get_account(proof_pda.0).await.unwrap().unwrap();
    let proof_ = Proof::try_from_bytes(&proof_account.data).unwrap();
    assert_eq!(proof_.authority, proof.authority);
    assert_eq!(proof_.claimable_rewards, 0);
    assert_eq!(proof_.hash, proof.hash);
    assert_eq!(proof_.total_hashes, proof.total_hashes);
    assert_eq!(proof_.total_rewards, proof.total_rewards);

    // Assert beneficiary state
    let beneficiary_account = banks
        .get_account(beneficiary_address)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(beneficiary_account.owner, spl_token::id());
    let beneficiary = spl_token::state::Account::unpack(&beneficiary_account.data).unwrap();
    assert_eq!(beneficiary.mint, ore::MINT_ADDRESS);
    assert_eq!(beneficiary.owner, payer.pubkey());
    assert_eq!(beneficiary.amount, amount);
    assert_eq!(beneficiary.delegate, COption::None);
    assert_eq!(beneficiary.state, AccountState::Initialized);
    assert_eq!(beneficiary.is_native, COption::None);
    assert_eq!(beneficiary.delegated_amount, 0);
    assert_eq!(beneficiary.close_authority, COption::None);
}

#[tokio::test]
async fn test_mine_alt_proof() {
    // Setup
    let (mut banks, payer, payer_alt, blockhash) = setup_program_test_env(true).await;

    // Submit register tx
    let proof_pda = Pubkey::find_program_address(&[PROOF, payer.pubkey().as_ref()], &ore::id());
    let ix = ore::instruction::register(payer.pubkey());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Submit register alt tx
    let proof_alt_pda =
        Pubkey::find_program_address(&[PROOF, payer_alt.pubkey().as_ref()], &ore::id());
    let ix_alt = ore::instruction::register(payer_alt.pubkey());
    let tx = Transaction::new_signed_with_payer(
        &[ix_alt],
        Some(&payer_alt.pubkey()),
        &[&payer_alt],
        blockhash,
    );
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Submit mine tx with invalid proof
    let proof_account = banks.get_account(proof_pda.0).await.unwrap().unwrap();
    let proof = Proof::try_from_bytes(&proof_account.data).unwrap();
    let (next_hash, nonce) = find_next_hash(
        proof.hash.into(),
        KeccakHash::new_from_array([u8::MAX; 32]),
        payer.pubkey(),
    );
    let ix = Instruction {
        program_id: ore::id(),
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(BUS_ADDRESSES[0], false),
            AccountMeta::new(proof_alt_pda.0, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
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
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_mine_correct_hash_alt_proof() {
    // Setup
    let (mut banks, payer, payer_alt, blockhash) = setup_program_test_env(true).await;

    // Submit register alt tx
    let proof_alt_pda =
        Pubkey::find_program_address(&[PROOF, payer_alt.pubkey().as_ref()], &ore::id());
    let ix_alt = ore::instruction::register(payer_alt.pubkey());
    let tx = Transaction::new_signed_with_payer(
        &[ix_alt],
        Some(&payer_alt.pubkey()),
        &[&payer_alt],
        blockhash,
    );
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Submit with correct hash for invalid proof
    let proof_alt_account = banks.get_account(proof_alt_pda.0).await.unwrap().unwrap();
    let proof_alt = Proof::try_from_bytes(&proof_alt_account.data).unwrap();
    let (next_hash, nonce) = find_next_hash(
        proof_alt.hash.into(),
        KeccakHash::new_from_array([u8::MAX; 32]),
        payer_alt.pubkey(),
    );
    let ix = Instruction {
        program_id: ore::id(),
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(BUS_ADDRESSES[0], false),
            AccountMeta::new(proof_alt_pda.0, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
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
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_mine_bus_rewards_insufficient() {
    // Setup
    let (mut banks, payer, _, blockhash) = setup_program_test_env(false).await;

    // Submit register tx
    let proof_pda = Pubkey::find_program_address(&[PROOF, payer.pubkey().as_ref()], &ore::id());
    let ix = ore::instruction::register(payer.pubkey());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Find next hash
    let proof_account = banks.get_account(proof_pda.0).await.unwrap().unwrap();
    let proof = Proof::try_from_bytes(&proof_account.data).unwrap();
    let (next_hash, nonce) = find_next_hash(
        proof.hash.into(),
        KeccakHash::new_from_array([u8::MAX; 32]),
        payer.pubkey(),
    );

    // Submit mine tx
    let ix = ore::instruction::mine(payer.pubkey(), BUS_ADDRESSES[0], next_hash.into(), nonce);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_claim_too_large() {
    // Setup
    let (mut banks, payer, _, blockhash) = setup_program_test_env(false).await;

    // Submit register tx
    let ix = ore::instruction::register(payer.pubkey());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Submit mine tx
    let beneficiary = get_associated_token_address(&payer.pubkey(), &ore::MINT_ADDRESS);
    let token_ix = create_associated_token_account(
        &payer.pubkey(),
        &payer.pubkey(),
        &ore::MINT_ADDRESS,
        &spl_token::id(),
    );
    let ix = ore::instruction::claim(payer.pubkey(), beneficiary, 1);
    let tx = Transaction::new_signed_with_payer(
        &[token_ix, ix],
        Some(&payer.pubkey()),
        &[&payer],
        blockhash,
    );
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_mine_fail_bad_data() {
    // Setup
    const FUZZ: usize = 10;
    let (mut banks, payer, _, blockhash) = setup_program_test_env(true).await;

    // Submit register tx
    let proof_pda = Pubkey::find_program_address(&[PROOF, payer.pubkey().as_ref()], &ore::id());
    let ix = ore::instruction::register(payer.pubkey());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Get proof
    let proof_account = banks.get_account(proof_pda.0).await.unwrap().unwrap();
    let proof = Proof::try_from_bytes(&proof_account.data).unwrap();

    // Shared variables for tests.
    let mut rng = rand::thread_rng();
    let (next_hash, nonce) = find_next_hash(
        proof.hash.into(),
        KeccakHash::new_from_array([u8::MAX; 32]),
        payer.pubkey(),
    );
    let signer = payer.pubkey();
    let proof_address = Pubkey::find_program_address(&[PROOF, signer.as_ref()], &ore::id()).0;

    // Fuzz randomized instruction data
    for _ in 0..FUZZ {
        let length_range = Uniform::from(5..=256);
        let length = rng.sample(length_range);
        let random_bytes: Vec<u8> = (0..length).map(|_| rng.gen()).collect();
        let ix = Instruction {
            program_id: ore::id(),
            accounts: vec![
                AccountMeta::new(signer, true),
                AccountMeta::new(BUS_ADDRESSES[0], false),
                AccountMeta::new(proof_address, false),
                AccountMeta::new(TREASURY_ADDRESS, false),
                AccountMeta::new_readonly(sysvar::slot_hashes::id(), false),
            ],
            data: [OreInstruction::Mine.to_vec(), random_bytes].concat(),
        };
        let tx =
            Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
        let res = banks.process_transaction(tx).await;
        assert!(res.is_err());
    }

    // Fuzz test random hashes and nonces
    for _ in 0..FUZZ {
        let next_hash = KeccakHash::new_unique();
        let nonce: u64 = rng.gen();
        assert_mine_tx_err(
            &mut banks,
            &payer,
            blockhash,
            payer.pubkey(),
            BUS_ADDRESSES[0],
            proof_address,
            TREASURY_ADDRESS,
            sysvar::slot_hashes::id(),
            next_hash,
            nonce,
        )
        .await;
    }

    // Fuzz test random bus addresses
    for _ in 0..FUZZ {
        assert_mine_tx_err(
            &mut banks,
            &payer,
            blockhash,
            payer.pubkey(),
            Pubkey::new_unique(),
            proof_address,
            TREASURY_ADDRESS,
            sysvar::slot_hashes::id(),
            next_hash,
            nonce,
        )
        .await;
    }

    // Fuzz test random proof addresses
    for _ in 0..FUZZ {
        assert_mine_tx_err(
            &mut banks,
            &payer,
            blockhash,
            payer.pubkey(),
            BUS_ADDRESSES[0],
            Pubkey::new_unique(),
            TREASURY_ADDRESS,
            sysvar::slot_hashes::id(),
            next_hash,
            nonce,
        )
        .await;
    }

    // Mix up the proof and treasury addresses
    assert_mine_tx_err(
        &mut banks,
        &payer,
        blockhash,
        payer.pubkey(),
        BUS_ADDRESSES[0],
        TREASURY_ADDRESS,
        proof_address,
        sysvar::slot_hashes::id(),
        next_hash,
        nonce,
    )
    .await;

    // Pass an invalid sysvar
    assert_mine_tx_err(
        &mut banks,
        &payer,
        blockhash,
        payer.pubkey(),
        BUS_ADDRESSES[0],
        proof_address,
        TREASURY_ADDRESS,
        sysvar::clock::id(),
        next_hash,
        nonce,
    )
    .await;
}

async fn assert_mine_tx_err(
    banks: &mut BanksClient,
    payer: &Keypair,
    blockhash: Hash,
    signer: Pubkey,
    bus: Pubkey,
    proof: Pubkey,
    treasury: Pubkey,
    slot_hash: Pubkey,
    next_hash: KeccakHash,
    nonce: u64,
) {
    let ix = Instruction {
        program_id: ore::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(bus, false),
            AccountMeta::new(proof, false),
            AccountMeta::new(treasury, false),
            AccountMeta::new_readonly(slot_hash, false),
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
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());
}

fn find_next_hash(hash: KeccakHash, difficulty: KeccakHash, signer: Pubkey) -> (KeccakHash, u64) {
    let mut next_hash: KeccakHash;
    let mut nonce = 0u64;
    loop {
        next_hash = hashv(&[
            hash.to_bytes().as_slice(),
            signer.to_bytes().as_slice(),
            nonce.to_le_bytes().as_slice(),
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

async fn setup_program_test_env(
    funded_busses: bool,
) -> (BanksClient, Keypair, Keypair, solana_program::hash::Hash) {
    let mut program_test = ProgramTest::new("ore", ore::ID, processor!(ore::process_instruction));
    program_test.prefer_bpf(true);

    // Busses
    for i in 0..BUS_COUNT {
        program_test.add_account_with_base64_data(
            BUS_ADDRESSES[i],
            1057920,
            ore::id(),
            bs64::encode(
                &[
                    &(Bus::discriminator() as u64).to_le_bytes(),
                    Bus {
                        id: i as u64,
                        rewards: if funded_busses { 250_000_000 } else { 0 },
                    }
                    .to_bytes(),
                ]
                .concat(),
            )
            .as_str(),
        );
    }

    // Treasury
    let admin_address = Pubkey::from_str("AeNqnoLwFanMd3ig9WoMxQZVwQHtCtqKMMBsT1sTrvz6").unwrap();
    let treasury_pda = Pubkey::find_program_address(&[TREASURY], &ore::id());
    program_test.add_account_with_base64_data(
        treasury_pda.0,
        1614720,
        ore::id(),
        bs64::encode(
            &[
                &(Treasury::discriminator() as u64).to_le_bytes(),
                Treasury {
                    bump: treasury_pda.1 as u64,
                    admin: admin_address,
                    difficulty: KeccakHash::new_from_array([u8::MAX; 32]).into(),
                    last_reset_at: START_AT,
                    reward_rate: INITIAL_REWARD_RATE,
                    total_claimed_rewards: 0,
                }
                .to_bytes(),
            ]
            .concat(),
        )
        .as_str(),
    );

    // Mint
    let mut mint_src: [u8; Mint::LEN] = [0; Mint::LEN];
    Mint {
        mint_authority: COption::Some(TREASURY_ADDRESS),
        supply: 2_000_000_000,
        decimals: TOKEN_DECIMALS,
        is_initialized: true,
        freeze_authority: COption::None,
    }
    .pack_into_slice(&mut mint_src);
    program_test.add_account_with_base64_data(
        MINT_ADDRESS,
        1461600,
        spl_token::id(),
        bs64::encode(&mint_src).as_str(),
    );

    // Treasury tokens
    let tokens_address = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &MINT_ADDRESS,
    );
    let mut tokens_src: [u8; spl_token::state::Account::LEN] = [0; spl_token::state::Account::LEN];
    spl_token::state::Account {
        mint: MINT_ADDRESS,
        owner: TREASURY_ADDRESS,
        amount: 2_000_000_000,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    }
    .pack_into_slice(&mut tokens_src);
    program_test.add_account_with_base64_data(
        tokens_address,
        2039280,
        spl_token::id(),
        bs64::encode(&tokens_src).as_str(),
    );

    // Set sysvar
    program_test.add_sysvar_account(
        sysvar::clock::id(),
        &Clock {
            slot: 0,
            epoch_start_timestamp: 0,
            epoch: 0,
            leader_schedule_epoch: DEFAULT_SLOTS_PER_EPOCH,
            unix_timestamp: START_AT + 1,
        },
    );

    let payer_alt = Keypair::new();
    program_test.add_account(
        payer_alt.pubkey(),
        Account {
            lamports: LAMPORTS_PER_SOL,
            data: vec![],
            owner: system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    let (banks, payer, blockhash) = program_test.start().await;
    (banks, payer, payer_alt, blockhash)
}
