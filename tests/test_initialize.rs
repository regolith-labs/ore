use mpl_token_metadata::{
    accounts::Metadata,
    types::{Key, TokenStandard},
};
use ore::{
    instruction::{InitializeArgs, OreInstruction},
    state::{Bus, Treasury},
    utils::AccountDeserialize,
    BUS, BUS_ADDRESSES, BUS_COUNT, INITIAL_DIFFICULTY, INITIAL_REWARD_RATE, METADATA,
    METADATA_ADDRESS, METADATA_NAME, METADATA_SYMBOL, METADATA_URI, MINT, MINT_ADDRESS, MINT_NOISE,
    TREASURY,
};
use solana_program::{
    hash::Hash, program_option::COption, program_pack::Pack, pubkey::Pubkey, rent::Rent,
};
use solana_program_test::{processor, read_file, BanksClient, ProgramTest};
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::state::{AccountState, Mint};

#[tokio::test]
async fn test_initialize() {
    // Setup
    let (mut banks, payer, blockhash) = setup_program_test_env().await;

    // Pdas
    let treasury_pda = Pubkey::find_program_address(&[TREASURY], &ore::id());
    let treasury_tokens_address =
        spl_associated_token_account::get_associated_token_address(&treasury_pda.0, &MINT_ADDRESS);

    // Submit tx
    let ix = ore::instruction::initialize(payer.pubkey());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Test bus state
    for i in 0..BUS_COUNT {
        let bus_account = banks.get_account(BUS_ADDRESSES[i]).await.unwrap().unwrap();
        assert_eq!(bus_account.owner, ore::id());
        let bus = Bus::try_from_bytes(&bus_account.data).unwrap();
        assert_eq!(bus.id as u8, i as u8);
        assert_eq!(bus.rewards, 0);
    }

    // Test treasury state
    let treasury_account = banks.get_account(treasury_pda.0).await.unwrap().unwrap();
    assert_eq!(treasury_account.owner, ore::id());
    let treasury = Treasury::try_from_bytes(&treasury_account.data).unwrap();
    assert_eq!(treasury.bump as u8, treasury_pda.1);
    assert_eq!(treasury.admin, payer.pubkey());
    assert_eq!(treasury.difficulty, INITIAL_DIFFICULTY.into());
    assert_eq!(treasury.last_reset_at as u8, 0);
    assert_eq!(treasury.reward_rate, INITIAL_REWARD_RATE);
    assert_eq!(treasury.total_claimed_rewards as u8, 0);

    // Test mint state
    let mint_account = banks.get_account(MINT_ADDRESS).await.unwrap().unwrap();
    assert_eq!(mint_account.owner, spl_token::id());
    let mint = Mint::unpack(&mint_account.data).unwrap();
    assert_eq!(mint.mint_authority, COption::Some(treasury_pda.0));
    assert_eq!(mint.supply, 0);
    assert_eq!(mint.decimals, ore::TOKEN_DECIMALS);
    assert_eq!(mint.is_initialized, true);
    assert_eq!(mint.freeze_authority, COption::None);

    // Test metadata state
    let metadata_account = banks.get_account(METADATA_ADDRESS).await.unwrap().unwrap();
    assert_eq!(metadata_account.owner, mpl_token_metadata::ID);
    let metadata = Metadata::from_bytes(&metadata_account.data).unwrap();
    assert_eq!(metadata.key, Key::MetadataV1);
    assert_eq!(metadata.update_authority, payer.pubkey());
    assert_eq!(metadata.mint, MINT_ADDRESS);
    assert_eq!(metadata.name.trim_end_matches('\0'), METADATA_NAME);
    assert_eq!(metadata.symbol.trim_end_matches('\0'), METADATA_SYMBOL);
    assert_eq!(metadata.uri.trim_end_matches('\0'), METADATA_URI);
    assert_eq!(metadata.seller_fee_basis_points, 0);
    assert_eq!(metadata.creators, None);
    assert_eq!(metadata.primary_sale_happened, false);
    assert_eq!(metadata.is_mutable, true);
    assert_eq!(metadata.token_standard, Some(TokenStandard::Fungible));
    assert_eq!(metadata.collection, None);
    assert_eq!(metadata.uses, None);
    assert_eq!(metadata.collection_details, None);
    assert_eq!(metadata.programmable_config, None);

    // Test treasury token state
    let treasury_tokens_account = banks
        .get_account(treasury_tokens_address)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(treasury_tokens_account.owner, spl_token::id());
    let treasury_tokens = spl_token::state::Account::unpack(&treasury_tokens_account.data).unwrap();
    assert_eq!(treasury_tokens.mint, MINT_ADDRESS);
    assert_eq!(treasury_tokens.owner, treasury_pda.0);
    assert_eq!(treasury_tokens.amount, 0);
    assert_eq!(treasury_tokens.delegate, COption::None);
    assert_eq!(treasury_tokens.state, AccountState::Initialized);
    assert_eq!(treasury_tokens.is_native, COption::None);
    assert_eq!(treasury_tokens.delegated_amount, 0);
    assert_eq!(treasury_tokens.close_authority, COption::None);
}

#[tokio::test]
async fn test_initialize_not_enough_accounts() {
    // Setup
    let (mut banks, payer, blockhash) = setup_program_test_env().await;

    // Submit tx
    let mut ix = ore::instruction::initialize(payer.pubkey());
    ix.accounts.remove(1);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_initialize_bad_pda() {
    // Setup
    let (mut banks, payer, blockhash) = setup_program_test_env().await;

    // Pdas
    let bus_pdas = [
        Pubkey::find_program_address(&[BUS, &[0]], &ore::id()),
        Pubkey::find_program_address(&[BUS, &[1]], &ore::id()),
        Pubkey::find_program_address(&[BUS, &[2]], &ore::id()),
        Pubkey::find_program_address(&[BUS, &[3]], &ore::id()),
        Pubkey::find_program_address(&[BUS, &[4]], &ore::id()),
        Pubkey::find_program_address(&[BUS, &[5]], &ore::id()),
        Pubkey::find_program_address(&[BUS, &[6]], &ore::id()),
        Pubkey::find_program_address(&[BUS, &[7]], &ore::id()),
    ];
    let mint_pda = Pubkey::find_program_address(&[MINT, MINT_NOISE.as_slice()], &ore::id());
    let treasury_pda = Pubkey::find_program_address(&[TREASURY], &ore::id());
    let metadata_pda = Pubkey::find_program_address(
        &[
            METADATA,
            mpl_token_metadata::ID.as_ref(),
            mint_pda.0.as_ref(),
        ],
        &mpl_token_metadata::ID,
    );
    let bad_pda = Pubkey::find_program_address(&[b"t"], &ore::id());

    // Bad bus pda
    let mut ix = ore::instruction::initialize(payer.pubkey());
    ix.accounts[1].pubkey = bad_pda.0;
    ix.data = [
        OreInstruction::Initialize.to_vec(),
        InitializeArgs {
            bus_0_bump: bad_pda.1,
            bus_1_bump: bus_pdas[1].1,
            bus_2_bump: bus_pdas[2].1,
            bus_3_bump: bus_pdas[3].1,
            bus_4_bump: bus_pdas[4].1,
            bus_5_bump: bus_pdas[5].1,
            bus_6_bump: bus_pdas[6].1,
            bus_7_bump: bus_pdas[7].1,
            metadata_bump: metadata_pda.1,
            mint_bump: mint_pda.1,
            treasury_bump: treasury_pda.1,
        }
        .to_bytes()
        .to_vec(),
    ]
    .concat();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());

    // Bad metadata pda
    let mut ix = ore::instruction::initialize(payer.pubkey());
    ix.accounts[9].pubkey = bad_pda.0;
    ix.data = [
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
            metadata_bump: bad_pda.1,
            mint_bump: mint_pda.1,
            treasury_bump: treasury_pda.1,
        }
        .to_bytes()
        .to_vec(),
    ]
    .concat();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());

    // Bad mint pda
    let mut ix = ore::instruction::initialize(payer.pubkey());
    ix.accounts[10].pubkey = bad_pda.0;
    ix.data = [
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
            metadata_bump: metadata_pda.1,
            mint_bump: bad_pda.1,
            treasury_bump: treasury_pda.1,
        }
        .to_bytes()
        .to_vec(),
    ]
    .concat();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());

    // Bad treasury pda
    let mut ix = ore::instruction::initialize(payer.pubkey());
    ix.accounts[11].pubkey = bad_pda.0;
    ix.data = [
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
            metadata_bump: metadata_pda.1,
            mint_bump: mint_pda.1,
            treasury_bump: bad_pda.1,
        }
        .to_bytes()
        .to_vec(),
    ]
    .concat();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());
}

async fn setup_program_test_env() -> (BanksClient, Keypair, Hash) {
    let mut program_test = ProgramTest::new("ore", ore::ID, processor!(ore::process_instruction));
    program_test.prefer_bpf(true);

    // Setup metadata program
    let data = read_file(&"tests/buffers/metadata_program.bpf");
    program_test.add_account(
        mpl_token_metadata::ID,
        Account {
            lamports: Rent::default().minimum_balance(data.len()).max(1),
            data,
            owner: solana_sdk::bpf_loader::id(),
            executable: true,
            rent_epoch: 0,
        },
    );

    program_test.start().await
}
