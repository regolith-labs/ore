use std::{mem::size_of, str::FromStr};

use ore::{
    state::{Bus, Proof, Treasury},
    utils::{AccountDeserialize, Discriminator},
    BUS_ADDRESSES, BUS_COUNT, INITIAL_REWARD_RATE, MINT_ADDRESS, PROOF, TOKEN_DECIMALS, TREASURY,
    TREASURY_ADDRESS,
};
use solana_program::{
    clock::Clock,
    epoch_schedule::DEFAULT_SLOTS_PER_EPOCH,
    keccak::{hashv, Hash as KeccakHash},
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    slot_hashes::SlotHash,
    sysvar,
};
use solana_program_test::{processor, BanksClient, ProgramTest};
use solana_sdk::{
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
    let (mut banks, payer, blockhash) = setup_program_test_env().await;

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
                        rewards: 250_000_000,
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
                    epoch_start_at: 100,
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
            slot: 10,
            epoch_start_timestamp: 0,
            epoch: 0,
            leader_schedule_epoch: DEFAULT_SLOTS_PER_EPOCH,
            unix_timestamp: 100,
        },
    );

    program_test.start().await
}
