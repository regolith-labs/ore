use ore::{
    instruction::{InitializeArgs, OreInstruction},
    state::{Bus, Treasury},
    BUS, BUS_COUNT, INITIAL_DIFFICULTY, INITIAL_REWARD_RATE, MINT, TREASURY,
};
use solana_program::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    system_program, sysvar,
};
use solana_program_test::{processor, BanksClient, ProgramTest};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::state::{AccountState, Mint};

#[tokio::test]
async fn test_initialize() {
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
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: [
            OreInstruction::Initialize.to_vec(),
            InitializeArgs {
                bus_0_bump: bus_pdas[0].1,
                bus_1_bump: bus_pdas[1].1,
                bus_2_bump: bus_pdas[2].1,
                bus_3_bump: bus_pdas[3].1,
                bus_4_bump: bus_pdas[4].1,
                bus_5_bump: bus_pdas[5].1,
                bus_6_bump: bus_pdas[6].1,
                bus_7_bump: bus_pdas[7].1,
                mint_bump: mint_pda.1,
                treasury_bump: treasury_pda.1,
            }
            .to_bytes()
            .to_vec(),
        ]
        .concat(),
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
        assert_eq!(bus.bump as u8, bus_pdas[i].1);
        assert_eq!(bus.id as u8, i as u8);
        assert_eq!(bus.available_rewards, 0);
        println!(
            "Bus {:?} {:?} {:?}",
            bus_pdas[i].0,
            bus_account,
            bs64::encode(&bus_account.data)
        );
    }

    // Test treasury state
    let treasury_account = banks.get_account(treasury_pda.0).await.unwrap().unwrap();
    assert_eq!(treasury_account.owner, ore::id());
    let treasury = Treasury::try_from_bytes(&treasury_account.data).unwrap();
    assert_eq!(treasury.bump as u8, treasury_pda.1);
    assert_eq!(treasury.admin, payer.pubkey());
    assert_eq!(treasury.difficulty, INITIAL_DIFFICULTY.into());
    assert_eq!(treasury.epoch_start_at as u8, 0);
    assert_eq!(treasury.reward_rate, INITIAL_REWARD_RATE);
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
    assert_eq!(mint.supply, 0);
    assert_eq!(mint.decimals, ore::TOKEN_DECIMALS);
    assert_eq!(mint.is_initialized, true);
    assert_eq!(mint.freeze_authority, COption::None);

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
    assert_eq!(treasury_tokens.amount, 0);
    assert_eq!(treasury_tokens.delegate, COption::None);
    assert_eq!(treasury_tokens.state, AccountState::Initialized);
    assert_eq!(treasury_tokens.is_native, COption::None);
    assert_eq!(treasury_tokens.delegated_amount, 0);
    assert_eq!(treasury_tokens.close_authority, COption::None);

    // assert!(false);
}

async fn setup_program_test_env() -> (BanksClient, Keypair, Hash) {
    let mut program_test = ProgramTest::new("ore", ore::ID, processor!(ore::process_instruction));
    program_test.prefer_bpf(true);
    program_test.start().await
}
