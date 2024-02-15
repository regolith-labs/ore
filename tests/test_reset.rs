use std::str::FromStr;

use ore::{
    instruction::OreInstruction,
    state::{Bus, Treasury},
    utils::AccountDeserialize,
    BUS, BUS_COUNT, BUS_EPOCH_REWARDS, INITIAL_DIFFICULTY, INITIAL_REWARD_RATE, MAX_EPOCH_REWARDS,
    MINT, TREASURY,
};
use solana_program::{
    clock::Clock,
    epoch_schedule::DEFAULT_SLOTS_PER_EPOCH,
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar,
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

    // Build ix
    let ix = Instruction {
        program_id: ore::ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(bus_pdas[0].0, false),
            AccountMeta::new(bus_pdas[1].0, false),
            AccountMeta::new(bus_pdas[2].0, false),
            AccountMeta::new(bus_pdas[3].0, false),
            AccountMeta::new(bus_pdas[4].0, false),
            AccountMeta::new(bus_pdas[5].0, false),
            AccountMeta::new(bus_pdas[6].0, false),
            AccountMeta::new(bus_pdas[7].0, false),
            AccountMeta::new(mint_pda.0, false),
            AccountMeta::new(treasury_pda.0, false),
            AccountMeta::new(treasury_tokens_address, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: [OreInstruction::Reset.to_vec()].concat(),
    };

    // Submit tx
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
        assert_eq!(bus.bump as u8, bus_pdas[i].1);
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
    // println!(
    //     "Mint {:?} {:?} {:?}",
    //     mint_pda.0,
    //     mint_account,
    //     bs64::encode(&mint_account.data)
    // );

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
    // println!(
    //     "Treasury tokens {:?} {:?} {:?}",
    //     treasury_tokens_address,
    //     treasury_tokens_account,
    //     bs64::encode(&treasury_tokens_account.data)
    // );

    // assert!(false);
}

async fn setup_program_test_env() -> (BanksClient, Keypair, Hash) {
    let mut program_test = ProgramTest::new("ore", ore::ID, processor!(ore::process_instruction));
    program_test.prefer_bpf(true);

    // Busses
    program_test.add_account_with_base64_data(
        Pubkey::from_str("2uwqyH2gKqstgAFCSniirx73X4iQek5ETc2vVJKUiNMg").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD/AAAAAAAAAAAAAAAAAAAA",
    );
    program_test.add_account_with_base64_data(
        Pubkey::from_str("FRMC6jVczm1cRaEs5EhDsfw7X8vsmSDpf3bJWVkawngu").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD+AAAAAQAAAAAAAAAAAAAA",
    );
    program_test.add_account_with_base64_data(
        Pubkey::from_str("9nWyycs4GHjnLujPR2sbA1A8K8CkiLc5VzxWUD4hg2uM").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD/AAAAAgAAAAAAAAAAAAAA",
    );
    program_test.add_account_with_base64_data(
        Pubkey::from_str("Kt7kqD3MyvxLbj4ek9urXUxkDoxaMuQn82K2VdYD1jM").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD6AAAAAwAAAAAAAAAAAAAA",
    );
    program_test.add_account_with_base64_data(
        Pubkey::from_str("8r9mXYnFQXhwrNfvatGUTxbbNSqxScuCwp4sBTSxDVTJ").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD9AAAABAAAAAAAAAAAAAAA",
    );
    program_test.add_account_with_base64_data(
        Pubkey::from_str("D9cEH32k8p9uWc4w5RrStK9rWssU8NuX1Dg5YaUim4wL").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD/AAAABQAAAAAAAAAAAAAA",
    );
    program_test.add_account_with_base64_data(
        Pubkey::from_str("H1RKMYADPzd4C1j1RZu51NvRSVktoTYEJyeVy98Kmdyu").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD/AAAABgAAAAAAAAAAAAAA",
    );
    program_test.add_account_with_base64_data(
        Pubkey::from_str("3XbdZNbBjjp8qnDJjv1RxaKisyfx6ahznYkSigs6dayy").unwrap(),
        1002240,
        ore::id(),
        "ZAAAAAAAAAD5AAAABwAAAAAAAAAAAAAA",
    );

    // Treasury
    program_test.add_account_with_base64_data(
        Pubkey::from_str("67PLJej6iZm915WbEu6NLeZtRZtnHc5nSVQvkHRZyPiC").unwrap(),
        1559040,
        ore::id(),
        "ZgAAAAAAAAD/AAAAAAAAAI9MXkItHZzhz/U8d4MsXPzDSQZSRgZsJnNpvgvcborrAAD///////////////////////////////////////8AAAAAAAAAAOgDAAAAAAAAAAAAAAAAAAA=",
    );

    // Mint
    program_test.add_account_with_base64_data(
        Pubkey::from_str("DY4JVebraRXg9BGt4MRU4mvqHGDzmi2Ay1HGjDU5YeNf").unwrap(),
        1461600,
        spl_token::id(),
        "AQAAAEvtK9pjA/sPMEl3rhUgX8iz4/q0A5icrVGp0GdL3satAAAAAAAAAAAJAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
    );

    // Treasury tokens
    program_test.add_account_with_base64_data(
        Pubkey::from_str("EH4tskvkeNqX5ce3FBr4oJob3FKSns9th7NvP28ZHsNL").unwrap(),
        2039280,
        spl_token::id(),
        "ukD7Oc0QjzbigRIB1x9/XLzAT3w7X0UTZ1NVeB85lRRL7SvaYwP7DzBJd64VIF/Is+P6tAOYnK1RqdBnS97GrQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
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
