use steel::*;

use crate::{instruction::*, state::*};

pub fn bet(
    signer: Pubkey,
    mint: Pubkey,
    amount: u64,
    id: u64,
    round: u64,
    seed: Option<[u8; 32]>,
) -> Instruction {
    let sender = spl_associated_token_account::get_associated_token_address(&signer, &mint);
    let block = block_pda().0;
    let block_bets = spl_associated_token_account::get_associated_token_address(&signer, &mint);
    let wager = wager_pda(round, id).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block, false),
            AccountMeta::new(block_bets, false),
            AccountMeta::new(sender, false),
            AccountMeta::new(wager, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(sysvar::slot_hashes::ID, false),
        ],
        data: Bet {
            amount: amount.to_le_bytes(),
            seed: seed.unwrap_or([0; 32]),
        }
        .to_bytes(),
    }
}
