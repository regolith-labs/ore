use spl_associated_token_account::get_associated_token_address;
use steel::*;

use crate::{
    consts::{BOOST_RESERVE_TOKEN, MARKET, MINT_ADDRESS, TREASURY_ADDRESS},
    instruction::*,
    state::*,
};

// let [signer_info, board_info, config_info, mint_info, treasury_info, vault_info, system_program, token_program, associated_token_program] =

pub fn initialize(signer: Pubkey) -> Instruction {
    let config_address = config_pda().0;
    let board_address = board_pda().0;
    let mint_address = MINT_ADDRESS;
    let treasury_address = TREASURY_ADDRESS;
    let vault_address = vault_address();
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(board_address, false),
            AccountMeta::new(config_address, false),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new(vault_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        ],
        data: Initialize {}.to_bytes(),
    }
}

// let [signer_info, square_info, system_program] = accounts else {

pub fn initialize_squares(signer: Pubkey) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new(square_pda(0).0, false),
            AccountMeta::new(square_pda(1).0, false),
            AccountMeta::new(square_pda(2).0, false),
            AccountMeta::new(square_pda(3).0, false),
            AccountMeta::new(square_pda(4).0, false),
            AccountMeta::new(square_pda(5).0, false),
            AccountMeta::new(square_pda(6).0, false),
            AccountMeta::new(square_pda(7).0, false),
            AccountMeta::new(square_pda(8).0, false),
            AccountMeta::new(square_pda(9).0, false),
            AccountMeta::new(square_pda(10).0, false),
            AccountMeta::new(square_pda(11).0, false),
            AccountMeta::new(square_pda(12).0, false),
            AccountMeta::new(square_pda(13).0, false),
            AccountMeta::new(square_pda(14).0, false),
            AccountMeta::new(square_pda(15).0, false),
            AccountMeta::new(square_pda(16).0, false),
            AccountMeta::new(square_pda(17).0, false),
            AccountMeta::new(square_pda(18).0, false),
            AccountMeta::new(square_pda(19).0, false),
            AccountMeta::new(square_pda(20).0, false),
            AccountMeta::new(square_pda(21).0, false),
            AccountMeta::new(square_pda(22).0, false),
            AccountMeta::new(square_pda(23).0, false),
            AccountMeta::new(square_pda(24).0, false),
        ],
        data: InitializeSquare {}.to_bytes(),
    }
}

pub fn claim(signer: Pubkey, amount: u64) -> Instruction {
    let miner_address = miner_pda(signer).0;
    let miner_tokens_address = get_associated_token_address(&miner_address, &MINT_ADDRESS);
    let recipient_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(miner_address, false),
            AccountMeta::new(miner_tokens_address, false),
            AccountMeta::new(recipient_address, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        ],
        data: Claim {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

// let [signer_info, board_info, mint_info, treasury_info, reserve_tokens_info, vault_info, system_program, token_program, slot_hashes_sysvar] =

pub fn reset(signer: Pubkey, miners: Vec<Pubkey>) -> Instruction {
    let board_address = board_pda().0;
    let mint_address = MINT_ADDRESS;
    let treasury_address = TREASURY_ADDRESS;
    let reserve_tokens_address = BOOST_RESERVE_TOKEN;
    let vault_address = vault_address();
    let mut accounts = vec![
        AccountMeta::new(signer, true),
        AccountMeta::new(board_address, false),
        AccountMeta::new(mint_address, false),
        AccountMeta::new(treasury_address, false),
        AccountMeta::new(reserve_tokens_address, false),
        AccountMeta::new(vault_address, false),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(sysvar::slot_hashes::ID, false),
    ];
    for miner in miners {
        if miner != Pubkey::default() {
            accounts.push(AccountMeta::new(miner_pda(miner).0, false));
        }
    }
    Instruction {
        program_id: crate::ID,
        accounts,
        data: Reset {}.to_bytes(),
    }
}

// let [signer_info, board_info, config_info, fee_collector_info, miner_info, mint_info, sender_info, square_info, vault_info, system_program, token_program, associated_token_program] =

pub fn prospect(signer: Pubkey, fee_collector: Pubkey, amount: u64, square_id: u64) -> Instruction {
    let board_address = board_pda().0;
    let config_address = config_pda().0;
    let miner_address = miner_pda(signer).0;
    let sender_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    let square_address = square_pda(square_id).0;
    let vault_address = vault_address();
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(board_address, false),
            AccountMeta::new(config_address, false),
            AccountMeta::new(fee_collector, false),
            AccountMeta::new(miner_address, false),
            AccountMeta::new(sender_address, false),
            AccountMeta::new(square_address, false),
            AccountMeta::new(vault_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: Prospect {
            amount: amount.to_le_bytes(),
            square_id: square_id.to_le_bytes(),
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

pub fn claim_seeker(signer: Pubkey, mint: Pubkey) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: ClaimSeeker {}.to_bytes(),
    }
}
