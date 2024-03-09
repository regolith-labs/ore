use mpl_token_metadata::{
    accounts::Metadata,
    types::{Key, TokenStandard},
};
use ore::{
    state::{Bus, Treasury},
    utils::AccountDeserialize,
    BUS_ADDRESSES, BUS_COUNT, INITIAL_DIFFICULTY, INITIAL_REWARD_RATE, METADATA_ADDRESS,
    METADATA_NAME, METADATA_SYMBOL, METADATA_URI, MINT_ADDRESS, TREASURY,
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
    assert_eq!(metadata.edition_nonce, Some(u8::MAX));
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
