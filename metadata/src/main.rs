use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::signer::EncodableKey;
use solana_sdk::transaction::Transaction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

fn main() {
    // Load the update authority keypair
    let update_authority = Keypair::read_from_file("/home/ubuntu/.config/solana/id.json").unwrap();
    let update_authority_pubkey = update_authority.pubkey();

    // Create the update metadata instruction
    let ix = mpl_token_metadata::instructions::UpdateMetadataAccountV2 {
        metadata: ore_api::consts::METADATA_ADDRESS,
        update_authority: update_authority_pubkey,
    }.instruction(
        mpl_token_metadata::instructions::UpdateMetadataAccountV2InstructionArgs {
            data: Some(mpl_token_metadata::types::DataV2 {
                name: "ORE".to_string(),
                symbol: "ORE".to_string(),
                uri: "https://ore.supply/assets/metadata.json".to_string(),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            }),
            new_update_authority: None,
            primary_sale_happened: None,
            is_mutable: None,
        }
    );

    // Create a Solana client
    let client = RpcClient::new_with_commitment(
        "https://api.mainnet-beta.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );

    // Create a transaction
    let mut transaction = Transaction::new_with_payer(
        &[ix],
        Some(&update_authority.pubkey()),
    );

    // Sign the transaction
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    transaction.sign(&[&update_authority], recent_blockhash);

    // Send the transaction
    // let signature = client.send_and_confirm_transaction(&transaction).unwrap();
    let result = client.simulate_transaction(&transaction).unwrap();
    println!("Sim result: {:?}", result);
}