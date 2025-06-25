use spl_associated_token_account::get_associated_token_address;
use steel::*;

use crate::{
    consts::{MINT_ADDRESS, TREASURY_ADDRESS},
    instruction::*,
    state::*,
};

pub fn open(signer: Pubkey, id: u64) -> Instruction {
    let block_adddress = block_pda(id).0;
    let config_address = config_pda().0;
    let market_address = market_pda(id).0;
    let base_mint_address = mint_pda(id).0;
    let collateral_address = collateral_pda(id).0;
    let commitment_address = commitment_pda(id).0;
    let sender_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    let vault_base_address = vault_base_pda(id).0;
    let vault_quote_address = vault_quote_pda(id).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(config_address, false),
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

pub fn close(signer: Pubkey, fee_collector: Pubkey, recipient: Pubkey, id: u64) -> Instruction {
    let block_adddress = block_pda(id).0;
    let config_address = config_pda().0;
    let collateral_address = collateral_pda(id).0;
    let commitment_address = commitment_pda(id).0;
    let fee_collector_address = get_associated_token_address(&fee_collector, &MINT_ADDRESS);
    let market_address = market_pda(id).0;
    let base_mint_address = mint_pda(id).0;
    let vault_base = vault_base_pda(id).0;
    let vault_quote = vault_quote_pda(id).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(config_address, false),
            AccountMeta::new(collateral_address, false),
            AccountMeta::new(commitment_address, false),
            AccountMeta::new(fee_collector_address, false),
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
    let commitment_address = commitment_pda(id).0;
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
    let commitment_address = commitment_pda(id).0;
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
    let collateral_address = collateral_pda(id).0;
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
    let tokens_base_address = get_associated_token_address(&signer, &base_mint_address);
    let tokens_quote_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    let vault_base_address = vault_base_pda(id).0;
    let vault_quote_address = vault_quote_pda(id).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            // AccountMeta::new(collateral_address, false),
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

pub fn set_admin(signer: Pubkey, admin: Pubkey) -> Instruction {
    let config_address = config_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(config_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: SetAdmin {
            admin: admin.to_bytes(),
        }
        .to_bytes(),
    }
}

pub fn set_block_limit(signer: Pubkey, block_limit: u64) -> Instruction {
    let config_address = config_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(config_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: SetBlockLimit {
            block_limit: block_limit.to_le_bytes(),
        }
        .to_bytes(),
    }
}

pub fn set_fee_collector(signer: Pubkey, fee_collector: Pubkey) -> Instruction {
    let config_address = config_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(config_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: SetFeeCollector {
            fee_collector: fee_collector.to_bytes(),
        }
        .to_bytes(),
    }
}

pub fn set_fee_rate(signer: Pubkey, fee_rate: u64) -> Instruction {
    let config_address = config_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(config_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: SetFeeRate {
            fee_rate: fee_rate.to_le_bytes(),
        }
        .to_bytes(),
    }
}
