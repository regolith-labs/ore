use ore::{state::Treasury, utils::AccountDeserialize, TREASURY_ADDRESS};
use solana_program::{hash::Hash, keccak::Hash as KeccakHash, rent::Rent};
use solana_program_test::{processor, read_file, BanksClient, ProgramTest};
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn test_update_difficulty() {
    // Setup
    let (mut banks, payer, blockhash) = setup_program_test_env().await;

    // Submit tx
    let ix = ore::instruction::initialize(payer.pubkey());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Get treasury account
    let treasury_account = banks.get_account(TREASURY_ADDRESS).await.unwrap().unwrap();
    let treasury = Treasury::try_from_bytes(&treasury_account.data).unwrap();

    // Submit update difficulty ix
    let new_difficulty = KeccakHash::new_unique();
    let ix = ore::instruction::update_difficulty(payer.pubkey(), new_difficulty.into());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Assert treasury state
    let treasury_account = banks.get_account(TREASURY_ADDRESS).await.unwrap().unwrap();
    let treasury_ = Treasury::try_from_bytes(&treasury_account.data).unwrap();
    assert_eq!(treasury_.bump, treasury.bump);
    assert_eq!(treasury_.admin, treasury.admin);
    assert_eq!(treasury_.difficulty, new_difficulty.into());
    assert_eq!(treasury_.last_reset_at, treasury.last_reset_at);
    assert_eq!(treasury_.reward_rate, treasury.reward_rate);
    assert_eq!(
        treasury_.total_claimed_rewards,
        treasury.total_claimed_rewards
    );
}

#[tokio::test]
async fn test_update_difficulty_bad_signer() {
    // Setup
    let (mut banks, payer, blockhash) = setup_program_test_env().await;

    // Submit tx
    let ix = ore::instruction::initialize(payer.pubkey());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Submit update difficulty ix
    let signer = Keypair::new();
    let new_difficulty = KeccakHash::new_unique();
    let ix = ore::instruction::update_difficulty(signer.pubkey(), new_difficulty.into());
    let tx =
        Transaction::new_signed_with_payer(&[ix], Some(&signer.pubkey()), &[&signer], blockhash);
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
