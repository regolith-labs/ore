use spl_associated_token_account::get_associated_token_address;
use steel::*;

use crate::{
    consts::{MARKET, MINT_ADDRESS, TREASURY_ADDRESS},
    instruction::*,
    state::*,
};

pub fn log(signer: Pubkey, msg: &[u8]) -> Instruction {
    let mut data = Log {}.to_bytes();
    data.extend_from_slice(msg);
    Instruction {
        program_id: crate::ID,
        accounts: vec![AccountMeta::new(signer, true)],
        data: data,
    }
}

pub fn program_log(accounts: &[AccountInfo], msg: &[u8]) -> Result<(), ProgramError> {
    invoke_signed(&log(*accounts[0].key, msg), accounts, &crate::ID, &[MARKET])
}

pub fn mine(signer: Pubkey, authority: Pubkey, id: u64, nonce: u64) -> Instruction {
    let block_adddress = block_pda(id).0;
    let market_address = market_pda().0;
    let miner_address = miner_pda(authority).0;
    let recipient = get_associated_token_address(&authority, &MINT_ADDRESS);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(authority, false),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(market_address, false),
            AccountMeta::new(miner_address, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(recipient, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(crate::ID, false),
            AccountMeta::new_readonly(sysvar::slot_hashes::ID, false),
        ],
        data: Mine {
            nonce: nonce.to_le_bytes(),
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
    let market_address = market_pda().0;
    let tokens_quote_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    let vault_address = vault_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(market_address, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(tokens_quote_address, false),
            AccountMeta::new(vault_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(crate::ID, false),
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
