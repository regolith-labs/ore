use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use steel::*;

use crate::{
    consts::{BOARD, MINT_ADDRESS, SOL_MINT, TREASURY_ADDRESS},
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

// let [signer_info, automation_info, executor_info, miner_info, system_program] = accounts else {

pub fn automate(
    signer: Pubkey,
    amount: u64,
    deposit: u64,
    executor: Pubkey,
    fee: u64,
    mask: u64,
    strategy: u8,
) -> Instruction {
    let automation_address = automation_pda(signer).0;
    let miner_address = miner_pda(signer).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(automation_address, false),
            AccountMeta::new(executor, false),
            AccountMeta::new(miner_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: Automate {
            amount: amount.to_le_bytes(),
            deposit: deposit.to_le_bytes(),
            fee: fee.to_le_bytes(),
            mask: mask.to_le_bytes(),
            strategy: strategy as u8,
        }
        .to_bytes(),
    }
}

pub fn claim_sol(signer: Pubkey) -> Instruction {
    let miner_address = miner_pda(signer).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(miner_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: ClaimSOL {}.to_bytes(),
    }
}

// let [signer_info, miner_info, mint_info, recipient_info, treasury_info, treasury_tokens_info, system_program, token_program, associated_token_program] =

pub fn claim_ore(signer: Pubkey) -> Instruction {
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
        data: ClaimORE {}.to_bytes(),
    }
}

// let [signer_info, authority_info, automation_info, board_info, miner_info, round_info, system_program] =

pub fn deploy(
    signer: Pubkey,
    authority: Pubkey,
    amount: u64,
    round_id: u64,
    squares: [bool; 25],
) -> Instruction {
    let automation_address = automation_pda(authority).0;
    let board_address = board_pda().0;
    let miner_address = miner_pda(authority).0;
    let round_address = round_pda(round_id).0;
    let entropy_var_address = entropy_api::state::var_pda(board_address, 0).0;

    // Convert array of 25 booleans into a 32-bit mask where each bit represents whether
    // that square index is selected (1) or not (0)
    let mut mask: u32 = 0;
    for (i, &square) in squares.iter().enumerate() {
        if square {
            mask |= 1 << i;
        }
    }

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(authority, false),
            AccountMeta::new(automation_address, false),
            AccountMeta::new(board_address, false),
            AccountMeta::new(miner_address, false),
            AccountMeta::new(round_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            // Entropy accounts.
            AccountMeta::new(entropy_var_address, false),
            AccountMeta::new_readonly(entropy_api::ID, false),
        ],
        data: Deploy {
            amount: amount.to_le_bytes(),
            squares: mask.to_le_bytes(),
        }
        .to_bytes(),
    }
}

// let [pool, user_source_token, user_destination_token, a_vault, b_vault, a_token_vault, b_token_vault, a_vault_lp_mint, b_vault_lp_mint, a_vault_lp, b_vault_lp, protocol_token_fee, user_key, vault_program, token_program] =

pub fn bury(signer: Pubkey, swap_accounts: &[AccountMeta], swap_data: &[u8]) -> Instruction {
    let board_address = board_pda().0;
    let config_address = config_pda().0;
    let mint_address = MINT_ADDRESS;
    let treasury_address = TREASURY_ADDRESS;
    let treasury_ore_address = get_associated_token_address(&treasury_address, &MINT_ADDRESS);
    let treasury_sol_address = get_associated_token_address(&treasury_address, &SOL_MINT);
    let mut accounts = vec![
        AccountMeta::new(signer, true),
        AccountMeta::new(board_address, false),
        AccountMeta::new_readonly(config_address, false),
        AccountMeta::new(mint_address, false),
        AccountMeta::new(treasury_address, false),
        AccountMeta::new(treasury_ore_address, false),
        AccountMeta::new(treasury_sol_address, false),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(crate::ID, false),
    ];
    for account in swap_accounts.iter() {
        let mut acc_clone = account.clone();
        acc_clone.is_signer = false;
        accounts.push(acc_clone);
    }
    let mut data = Bury {}.to_bytes();
    data.extend_from_slice(swap_data);
    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn wrap(signer: Pubkey) -> Instruction {
    let config_address = config_pda().0;
    let treasury_address = TREASURY_ADDRESS;
    let treasury_sol_address = get_associated_token_address(&treasury_address, &SOL_MINT);
    Instruction {
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new_readonly(config_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new(treasury_sol_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        program_id: crate::ID,
        data: Wrap {}.to_bytes(),
    }
}

// let [signer_info, board_info, config_info, fee_collector_info, mint_info, round_info, round_next_info, top_miner_info, treasury_info, treasury_tokens_info, system_program, token_program, ore_program, slot_hashes_sysvar] =

pub fn reset(
    signer: Pubkey,
    fee_collector: Pubkey,
    round_id: u64,
    top_miner: Pubkey,
) -> Instruction {
    let board_address = board_pda().0;
    let config_address = config_pda().0;
    let mint_address = MINT_ADDRESS;
    let round_address = round_pda(round_id).0;
    let round_next_address = round_pda(round_id + 1).0;
    let top_miner_address = miner_pda(top_miner).0;
    let treasury_address = TREASURY_ADDRESS;
    let treasury_tokens_address = treasury_tokens_address();
    let entropy_var_address = entropy_api::state::var_pda(board_address, 0).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(board_address, false),
            AccountMeta::new(config_address, false),
            AccountMeta::new(fee_collector, false),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(round_address, false),
            AccountMeta::new(round_next_address, false),
            AccountMeta::new(top_miner_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new(treasury_tokens_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(crate::ID, false),
            AccountMeta::new_readonly(sysvar::slot_hashes::ID, false),
            // Entropy accounts.
            AccountMeta::new(entropy_var_address, false),
            AccountMeta::new_readonly(entropy_api::ID, false),
        ],
        data: Reset {}.to_bytes(),
    }
}

// let [signer_info, board_info, rent_payer_info, round_info, treasury_info, system_program] =

pub fn close(signer: Pubkey, round_id: u64, rent_payer: Pubkey) -> Instruction {
    let board_address = board_pda().0;
    let treasury_address = TREASURY_ADDRESS;
    let round_address = round_pda(round_id).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(board_address, false),
            AccountMeta::new(rent_payer, false),
            AccountMeta::new(round_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: Close {}.to_bytes(),
    }
}

// let [signer_info, automation_info, board_info, miner_info, round_info, treasury_info, system_program] =

pub fn checkpoint(signer: Pubkey, authority: Pubkey, round_id: u64) -> Instruction {
    let miner_address = miner_pda(authority).0;
    let board_address = board_pda().0;
    let round_address = round_pda(round_id).0;
    let treasury_address = TREASURY_ADDRESS;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(board_address, false),
            AccountMeta::new(miner_address, false),
            AccountMeta::new(round_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: Checkpoint {}.to_bytes(),
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

pub fn set_buffer(signer: Pubkey, buffer: u64) -> Instruction {
    let config_address = config_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(config_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: SetBuffer {
            buffer: buffer.to_le_bytes(),
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

// let [signer_info, mint_info, sender_info, stake_info, stake_tokens_info, treasury_info, system_program, token_program, associated_token_program] =

pub fn deposit(signer: Pubkey, amount: u64) -> Instruction {
    let mint_address = MINT_ADDRESS;
    let stake_address = stake_pda(signer).0;
    let stake_tokens_address = get_associated_token_address(&stake_address, &MINT_ADDRESS);
    let sender_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    let treasury_address = TREASURY_ADDRESS;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(sender_address, false),
            AccountMeta::new(stake_address, false),
            AccountMeta::new(stake_tokens_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        ],
        data: Deposit {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

// let [signer_info, mint_info, recipient_info, stake_info, stake_tokens_info, treasury_info, system_program, token_program, associated_token_program] =

pub fn withdraw(signer: Pubkey, amount: u64) -> Instruction {
    let stake_address = stake_pda(signer).0;
    let stake_tokens_address = get_associated_token_address(&stake_address, &MINT_ADDRESS);
    let mint_address = MINT_ADDRESS;
    let recipient_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    let treasury_address = TREASURY_ADDRESS;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(recipient_address, false),
            AccountMeta::new(stake_address, false),
            AccountMeta::new(stake_tokens_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        ],
        data: Withdraw {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

// let [signer_info, mint_info, recipient_info, stake_info, treasury_info, treasury_tokens_info, system_program, token_program, associated_token_program] =

pub fn claim_yield(signer: Pubkey, amount: u64) -> Instruction {
    let stake_address = stake_pda(signer).0;
    let mint_address = MINT_ADDRESS;
    let recipient_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    let treasury_address = TREASURY_ADDRESS;
    let treasury_tokens_address = treasury_tokens_address();
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(recipient_address, false),
            AccountMeta::new(stake_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new(treasury_tokens_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        ],
        data: ClaimYield {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

pub fn new_var(
    signer: Pubkey,
    provider: Pubkey,
    id: u64,
    commit: [u8; 32],
    samples: u64,
) -> Instruction {
    let board_address = board_pda().0;
    let config_address = config_pda().0;
    let var_address = entropy_api::state::var_pda(board_address, id).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(board_address, false),
            AccountMeta::new(config_address, false),
            AccountMeta::new(provider, false),
            AccountMeta::new(var_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(entropy_api::ID, false),
        ],
        data: NewVar {
            id: id.to_le_bytes(),
            commit: commit,
            samples: samples.to_le_bytes(),
        }
        .to_bytes(),
    }
}

pub fn set_swap_program(signer: Pubkey, new_program: Pubkey) -> Instruction {
    let config_address = config_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(config_address, false),
            AccountMeta::new_readonly(new_program, false),
        ],
        data: SetSwapProgram {}.to_bytes(),
    }
}
