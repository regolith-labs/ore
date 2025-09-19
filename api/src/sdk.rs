use spl_associated_token_account::get_associated_token_address;
use steel::*;

use crate::{
    consts::{BOARD, BOOST_RESERVE_TOKEN, MINT_ADDRESS, TREASURY_ADDRESS},
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
    invoke_signed(&log(*accounts[0].key, msg), accounts, &crate::ID, &[BOARD])
}

// let [signer_info, config_info, mint_info, reserve_tokens_info, treasury_info, system_program, token_program] =

pub fn boost(signer: Pubkey) -> Instruction {
    let config_address = config_pda().0;
    let mint_address = MINT_ADDRESS;
    let reserve_tokens_address = BOOST_RESERVE_TOKEN;
    let treasury_address = TREASURY_ADDRESS;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(config_address, false),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(reserve_tokens_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: Boost {}.to_bytes(),
    }
}

// let [signer_info, board_info, config_info, mint_info, treasury_info, vault_info, system_program, token_program, associated_token_program] =

pub fn initialize(signer: Pubkey) -> Instruction {
    let config_address = config_pda().0;
    let board_address = board_pda().0;
    let mint_address = MINT_ADDRESS;
    let treasury_address = TREASURY_ADDRESS;
    let treasury_tokens_address = treasury_tokens_address();
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(board_address, false),
            AccountMeta::new(config_address, false),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new(treasury_tokens_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        ],
        data: Initialize {}.to_bytes(),
    }
}

pub fn claim_sol(signer: Pubkey, amount: u64) -> Instruction {
    let miner_address = miner_pda(signer).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(miner_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: ClaimSOL {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

// let [signer_info, miner_info, mint_info, recipient_info, treasury_info, treasury_tokens_info, system_program, token_program, associated_token_program] =

pub fn claim_ore(signer: Pubkey, amount: u64) -> Instruction {
    let miner_address = miner_pda(signer).0;
    let treasury_address = treasury_pda().0;
    let treasury_tokens_address = get_associated_token_address(&treasury_address, &MINT_ADDRESS);
    let recipient_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(miner_address, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(recipient_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new(treasury_tokens_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        ],
        data: ClaimORE {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

pub fn redeem(signer: Pubkey, amount: u64) -> Instruction {
    let mint_address = MINT_ADDRESS;
    let sender_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    let treasury_address = TREASURY_ADDRESS;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(sender_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: Redeem {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

// let [signer_info, board_info, mint_info, treasury_info, treasury_tokens_info, system_program, token_program, slot_hashes_sysvar] =

pub fn reset(signer: Pubkey, miners: Vec<Pubkey>) -> Instruction {
    let board_address = board_pda().0;
    let config_address = config_pda().0;
    let mint_address = MINT_ADDRESS;
    let square_address = square_pda().0;
    let treasury_address = TREASURY_ADDRESS;
    let treasury_tokens_address = treasury_tokens_address();
    let mut accounts = vec![
        AccountMeta::new(signer, true),
        AccountMeta::new(board_address, false),
        AccountMeta::new(config_address, false),
        AccountMeta::new(mint_address, false),
        AccountMeta::new(square_address, false),
        AccountMeta::new(treasury_address, false),
        AccountMeta::new(treasury_tokens_address, false),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(crate::ID, false),
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

// let [signer_info, board_info, config_info, fee_collector_info, miner_info, sender_info, square_info, system_program] =

pub fn prospect(signer: Pubkey, fee_collector: Pubkey, amount: u64, square_id: u64) -> Instruction {
    let board_address = board_pda().0;
    let config_address = config_pda().0;
    let miner_address = miner_pda(signer).0;
    let square_address = square_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(board_address, false),
            AccountMeta::new(config_address, false),
            AccountMeta::new(fee_collector, false),
            AccountMeta::new(miner_address, false),
            AccountMeta::new(square_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
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
