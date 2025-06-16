use spl_associated_token_account::get_associated_token_address;
use steel::*;

use crate::{
    consts::{MINT_ADDRESS, TREASURY_ADDRESS},
    instruction::*,
    state::*,
};

pub fn open(signer: Pubkey, id: u64) -> Instruction {
    let block_adddress = block_pda(id).0;
    let market_address = market_pda(id).0;
    let base_mint_address = mint_pda(id).0;
    let collateral_address = get_associated_token_address(&block_adddress, &MINT_ADDRESS);
    let commitment_address = get_associated_token_address(&block_adddress, &base_mint_address);
    let sender_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    let vault_base_address = get_associated_token_address(&market_address, &base_mint_address);
    let vault_quote_address = get_associated_token_address(&market_address, &MINT_ADDRESS);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(collateral_address, false),
            AccountMeta::new(commitment_address, false),
            AccountMeta::new(market_address, false),
            AccountMeta::new(base_mint_address, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(sender_address, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new(vault_base_address, false),
            AccountMeta::new(vault_quote_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
        ],
        data: Open {
            id: id.to_le_bytes(),
        }
        .to_bytes(),
    }
}

pub fn close(signer: Pubkey, recipient: Pubkey, id: u64) -> Instruction {
    let block_adddress = block_pda(id).0;
    let market_address = market_pda(id).0;
    let base_mint_address = mint_pda(id).0;
    let vault_base = get_associated_token_address(&market_address, &base_mint_address);
    let vault_quote = get_associated_token_address(&market_address, &MINT_ADDRESS);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(market_address, false),
            AccountMeta::new(base_mint_address, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(recipient, false),
            AccountMeta::new_readonly(TREASURY_ADDRESS, false),
            AccountMeta::new(vault_base, false),
            AccountMeta::new(vault_quote, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: Close {}.to_bytes(),
    }
}

pub fn mine(signer: Pubkey, id: u64, amount: u64) -> Instruction {
    let block_adddress = block_pda(id).0;
    let market_address = market_pda(id).0;
    let base_mint_address = mint_pda(id).0;
    let miner_address = miner_pda(signer).0;
    let sender = get_associated_token_address(&signer, &base_mint_address);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(market_address, false),
            AccountMeta::new(miner_address, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(sender, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(sysvar::slot_hashes::ID, false),
        ],
        data: Mine {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

// let [signer_info, block_info, commitment_info, market_info, miner_info, mint_info, permit_info, sender_info, system_program, token_program, slot_hashes_sysvar] =

pub fn commit(
    signer: Pubkey,
    amount: u64,
    executor: Pubkey,
    fee: u64,
    id: u64,
    seed: [u8; 32],
) -> Instruction {
    let block_adddress = block_pda(id).0;
    let market_address = market_pda(id).0;
    let base_mint_address = mint_pda(id).0;
    let miner_address = miner_pda(signer).0;
    let permit_address = permit_pda(signer, id).0;
    let commitment_address = get_associated_token_address(&block_adddress, &base_mint_address);
    let sender_address = get_associated_token_address(&signer, &base_mint_address);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(commitment_address, false),
            AccountMeta::new(market_address, false),
            AccountMeta::new(miner_address, false),
            AccountMeta::new(base_mint_address, false),
            AccountMeta::new(permit_address, false),
            AccountMeta::new(sender_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: Commit {
            amount: amount.to_le_bytes(),
            executor: executor.to_bytes(),
            fee: fee.to_le_bytes(),
            seed: seed,
        }
        .to_bytes(),
    }
}

pub fn uncommit(signer: Pubkey, amount: u64, id: u64) -> Instruction {
    let block_adddress = block_pda(id).0;
    let market_address = market_pda(id).0;
    let base_mint_address = mint_pda(id).0;
    let miner_address = miner_pda(signer).0;
    let permit_address = permit_pda(signer, id).0;
    let commitment_address = get_associated_token_address(&block_adddress, &base_mint_address);
    let recipient_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(commitment_address, false),
            AccountMeta::new(market_address, false),
            AccountMeta::new(miner_address, false),
            AccountMeta::new(base_mint_address, false),
            AccountMeta::new(permit_address, false),
            AccountMeta::new(recipient_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: Uncommit {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

pub fn deposit(signer: Pubkey, id: u64, amount: u64) -> Instruction {
    let block_adddress = block_pda(id).0;
    let collateral_address = get_associated_token_address(&block_adddress, &MINT_ADDRESS);
    let stake_address = stake_pda(signer, id).0;
    let sender = get_associated_token_address(&signer, &MINT_ADDRESS);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(collateral_address, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(sender, false),
            AccountMeta::new(stake_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: Deposit {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

// let [signer_info, block_info, collateral_info, mint_ore_info, sender_info, stake_info, system_program, token_program] =

// let [signer_info, block_info, collateral_info, market_info, mint_base_info, mint_quote_info, stake_info, tokens_base_info, tokens_quote_info, vault_base_info, vault_quote_info, system_program, token_program, associated_token_program] =

pub fn swap(
    signer: Pubkey,
    id: u64,
    amount: u64,
    direction: SwapDirection,
    precision: SwapPrecision,
) -> Instruction {
    let block_adddress = block_pda(id).0;
    let market_address = market_pda(id).0;
    let base_mint_address = mint_pda(id).0;
    let stake_address = stake_pda(signer, id).0;
    let collateral_address = get_associated_token_address(&block_adddress, &MINT_ADDRESS);
    let tokens_base_address = get_associated_token_address(&signer, &base_mint_address);
    let tokens_quote_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    let vault_base_address = get_associated_token_address(&market_address, &base_mint_address);
    let vault_quote_address = get_associated_token_address(&market_address, &MINT_ADDRESS);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(collateral_address, false),
            AccountMeta::new(market_address, false),
            AccountMeta::new(base_mint_address, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(stake_address, false),
            AccountMeta::new(tokens_base_address, false),
            AccountMeta::new(tokens_quote_address, false),
            AccountMeta::new(vault_base_address, false),
            AccountMeta::new(vault_quote_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        ],
        data: Swap {
            amount: amount.to_le_bytes(),
            direction: direction as u8,
            precision: precision as u8,
        }
        .to_bytes(),
    }
}
