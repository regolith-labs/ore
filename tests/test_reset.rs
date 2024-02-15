use std::str::FromStr;

use ore::{
    state::{Bus, Treasury},
    utils::{AccountDeserialize, Discriminator},
    BUS, BUS_ADDRESSES, BUS_COUNT, BUS_EPOCH_REWARDS, INITIAL_DIFFICULTY, INITIAL_REWARD_RATE,
    MAX_EPOCH_REWARDS, MINT, MINT_ADDRESS, TOKEN_DECIMALS, TREASURY, TREASURY_ADDRESS,
};
use solana_program::{
    clock::Clock, epoch_schedule::DEFAULT_SLOTS_PER_EPOCH, hash::Hash, program_option::COption,
    program_pack::Pack, pubkey::Pubkey, sysvar,
};
use solana_program_test::{processor, BanksClient, ProgramTest};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::state::{AccountState, Mint};

#[tokio::test]
async fn test_reset() {
    // Setup
    let (mut banks, payer, hash) = setup_program_test_env().await;

    // Pdas
    let bus_pdas = vec![
        Pubkey::find_program_address(&[BUS, &[0]], &ore::id()),
        Pubkey::find_program_address(&[BUS, &[1]], &ore::id()),
        Pubkey::find_program_address(&[BUS, &[2]], &ore::id()),
        Pubkey::find_program_address(&[BUS, &[3]], &ore::id()),
        Pubkey::find_program_address(&[BUS, &[4]], &ore::id()),
        Pubkey::find_program_address(&[BUS, &[5]], &ore::id()),
        Pubkey::find_program_address(&[BUS, &[6]], &ore::id()),
        Pubkey::find_program_address(&[BUS, &[7]], &ore::id()),
    ];
    let mint_pda = Pubkey::find_program_address(&[MINT], &ore::id());
    let treasury_pda = Pubkey::find_program_address(&[TREASURY], &ore::id());
    let treasury_tokens_address =
        spl_associated_token_account::get_associated_token_address(&treasury_pda.0, &mint_pda.0);

    // Submit tx
    let ix = ore::instruction::reset(payer.pubkey());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], hash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Test bus state
    for i in 0..BUS_COUNT {
        let bus_account = banks.get_account(bus_pdas[i].0).await.unwrap().unwrap();
        assert_eq!(bus_account.owner, ore::id());
        let bus = Bus::try_from_bytes(&bus_account.data).unwrap();
        println!(
            "Bus {:?} {:?} {:?}",
            bus_pdas[i].0,
            bus_account,
            bs64::encode(&bus_account.data)
        );
        assert_eq!(bus.id as u8, i as u8);
        assert_eq!(bus.available_rewards, BUS_EPOCH_REWARDS);
    }

    // Test treasury state
    let treasury_account = banks.get_account(treasury_pda.0).await.unwrap().unwrap();
    assert_eq!(treasury_account.owner, ore::id());
    let treasury = Treasury::try_from_bytes(&treasury_account.data).unwrap();
    assert_eq!(treasury.bump as u8, treasury_pda.1);
    assert_eq!(
        treasury.admin,
        Pubkey::from_str("AeNqnoLwFanMd3ig9WoMxQZVwQHtCtqKMMBsT1sTrvz6").unwrap()
    );
    assert_eq!(treasury.difficulty, INITIAL_DIFFICULTY.into());
    assert_eq!(treasury.epoch_start_at as u8, 100);
    assert_eq!(treasury.reward_rate, INITIAL_REWARD_RATE.saturating_div(2));
    assert_eq!(treasury.total_claimed_rewards as u8, 0);
    println!(
        "Treasury {:?} {:?} {:?}",
        treasury_pda.0,
        treasury_account,
        bs64::encode(&treasury_account.data)
    );

    // Test mint state
    let mint_account = banks.get_account(mint_pda.0).await.unwrap().unwrap();
    assert_eq!(mint_account.owner, spl_token::id());
    let mint = Mint::unpack(&mint_account.data).unwrap();
    assert_eq!(mint.mint_authority, COption::Some(treasury_pda.0));
    assert_eq!(mint.supply, MAX_EPOCH_REWARDS);
    assert_eq!(mint.decimals, ore::TOKEN_DECIMALS);
    assert_eq!(mint.is_initialized, true);
    assert_eq!(mint.freeze_authority, COption::None);
    println!(
        "Mint {:?} {:?} {:?}",
        mint_pda.0,
        mint_account,
        bs64::encode(&mint_account.data)
    );

    // Test treasury token state
    let treasury_tokens_account = banks
        .get_account(treasury_tokens_address)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(treasury_tokens_account.owner, spl_token::id());
    let treasury_tokens = spl_token::state::Account::unpack(&treasury_tokens_account.data).unwrap();
    assert_eq!(treasury_tokens.mint, mint_pda.0);
    assert_eq!(treasury_tokens.owner, treasury_pda.0);
    assert_eq!(treasury_tokens.amount, MAX_EPOCH_REWARDS);
    assert_eq!(treasury_tokens.delegate, COption::None);
    assert_eq!(treasury_tokens.state, AccountState::Initialized);
    assert_eq!(treasury_tokens.is_native, COption::None);
    assert_eq!(treasury_tokens.delegated_amount, 0);
    assert_eq!(treasury_tokens.close_authority, COption::None);
    println!(
        "Treasury tokens {:?} {:?} {:?}",
        treasury_tokens_address,
        treasury_tokens_account,
        bs64::encode(&treasury_tokens_account.data)
    );

    // assert!(false);
}

async fn setup_program_test_env() -> (BanksClient, Keypair, Hash) {
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
                        available_rewards: 0,
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
                    difficulty: INITIAL_DIFFICULTY.into(),
                    epoch_start_at: 0,
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
        supply: 0,
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
        amount: 0,
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
