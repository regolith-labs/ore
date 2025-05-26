use meteora_pools_sdk::instructions::Swap;
use spl_token::native_mint;
use steel::*;

use crate::{
    consts::{MINT_ADDRESS, TREASURY_ADDRESS, TREASURY_TOKENS_ADDRESS},
    instruction::*,
    state::*,
};

pub fn bet(signer: Pubkey, mint: Pubkey, amount: u64, round: u64, seed: [u8; 32]) -> Instruction {
    let sender = spl_associated_token_account::get_associated_token_address(&signer, &mint);
    let block = block_pda().0;
    let block_bets = spl_associated_token_account::get_associated_token_address(&block, &mint);
    let wager = wager_pda(round, seed).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block, false),
            AccountMeta::new(block_bets, false),
            AccountMeta::new(sender, false),
            AccountMeta::new(wager, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(sysvar::slot_hashes::ID, false),
        ],
        data: Bet {
            amount: amount.to_le_bytes(),
            seed,
        }
        .to_bytes(),
    }
}

pub fn bury(signer: Pubkey, swap: Swap) -> Instruction {
    let block = block_pda().0;
    let block_bets = spl_associated_token_account::get_associated_token_address(
        &block,
        &spl_token::native_mint::ID,
    );
    let block_ore =
        spl_associated_token_account::get_associated_token_address(&block, &MINT_ADDRESS);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            // required accounts
            AccountMeta::new(signer, true),
            AccountMeta::new(block, false),
            AccountMeta::new(block_bets, false),
            AccountMeta::new(block_ore, false),
            AccountMeta::new(spl_token::native_mint::ID, false),
            AccountMeta::new(MINT_ADDRESS, false),
            // swap accounts
            AccountMeta::new(swap.pool, false),
            AccountMeta::new(swap.a_vault, false),
            AccountMeta::new(swap.b_vault, false),
            AccountMeta::new(swap.a_token_vault, false),
            AccountMeta::new(swap.b_token_vault, false),
            AccountMeta::new(swap.a_vault_lp_mint, false),
            AccountMeta::new(swap.b_vault_lp_mint, false),
            AccountMeta::new(swap.a_vault_lp, false),
            AccountMeta::new(swap.b_vault_lp, false),
            AccountMeta::new(swap.protocol_token_fee, false),
            AccountMeta::new_readonly(swap.vault_program, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(meteora_pools_sdk::programs::AMM_ID, false),
        ],
        data: Bury {}.to_bytes(),
    }
}

pub fn close(signer: Pubkey, wager: Pubkey) -> Instruction {
    let block = block_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block, false),
            AccountMeta::new(wager, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: Close {}.to_bytes(),
    }
}

pub fn initialize(signer: Pubkey) -> Instruction {
    let block = block_pda().0;
    let block_bets =
        spl_associated_token_account::get_associated_token_address(&block, &native_mint::ID);
    let block_ore =
        spl_associated_token_account::get_associated_token_address(&block, &MINT_ADDRESS);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block, false),
            AccountMeta::new(block_bets, false),
            AccountMeta::new(block_ore, false),
            AccountMeta::new_readonly(MINT_ADDRESS, false),
            AccountMeta::new_readonly(native_mint::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        ],
        data: Initialize {}.to_bytes(),
    }
}

pub fn payout(signer: Pubkey, wager: Pubkey, recipient: Pubkey) -> Instruction {
    let block = block_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(wager, false),
            AccountMeta::new(recipient, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new(TREASURY_TOKENS_ADDRESS, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(sysvar::slot_hashes::ID, false),
        ],
        data: Payout {}.to_bytes(),
    }
}

pub fn reset(signer: Pubkey, boost_config: Pubkey) -> Instruction {
    let block = block_pda().0;
    let boost_proof = proof_pda(boost_config).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new(TREASURY_TOKENS_ADDRESS, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(boost_config, false),
            AccountMeta::new(boost_proof, false),
        ],
        data: Reset {}.to_bytes(),
    }
}
