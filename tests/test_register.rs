use std::str::FromStr;

use ore::{
    instruction::{register, OreInstruction, RegisterArgs},
    state::{Bus, Treasury},
    utils::Discriminator,
    BUS_ADDRESSES, BUS_COUNT, INITIAL_REWARD_RATE, MINT_ADDRESS, PROOF, TOKEN_DECIMALS, TREASURY,
    TREASURY_ADDRESS,
};
use solana_program::{
    clock::Clock,
    epoch_schedule::DEFAULT_SLOTS_PER_EPOCH,
    instruction::{AccountMeta, Instruction},
    keccak::Hash as KeccakHash,
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    sysvar,
};
use solana_program_test::{processor, BanksClient, ProgramTest};
use solana_sdk::{
    signature::{Keypair, Signer},
    system_transaction::transfer,
    transaction::Transaction,
};
use spl_token::state::{AccountState, Mint};

#[tokio::test]
async fn test_register_account_with_lamports() {
    let (mut banks, payer, blockhash) = setup_program_test_env().await;

    // Send lamports to the proof pda
    let proof_pda = Pubkey::find_program_address(&[PROOF, payer.pubkey().as_ref()], &ore::id());
    let lamports = Rent::default().minimum_balance(0);
    let tx = transfer(&payer, &proof_pda.0, lamports, blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Assert register succeeds
    let ix = register(payer.pubkey());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_register_fail_other() {
    let (mut banks, payer, blockhash) = setup_program_test_env().await;

    // Try register for another keypair
    let other = Keypair::new();
    let proof_pda = Pubkey::find_program_address(&[PROOF, other.pubkey().as_ref()], &ore::id());
    let ix = Instruction {
        program_id: ore::id(),
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(proof_pda.0, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: [
            OreInstruction::Register.to_vec(),
            RegisterArgs { bump: proof_pda.1 }.to_bytes().to_vec(),
        ]
        .concat(),
    };
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());
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
                    last_reset_at: 100,
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
