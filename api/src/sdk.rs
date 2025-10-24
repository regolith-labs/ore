use solana_program::pubkey;
use solana_program::pubkey::Pubkey;
use spl_associated_token_account::{
    get_associated_token_address, get_associated_token_address_with_program_id,
};
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
        ],
        data: Deploy {
            amount: amount.to_le_bytes(),
            squares: mask.to_le_bytes(),
        }
        .to_bytes(),
    }
}

const POOL_ADDRESS: Pubkey = pubkey!("GgaDTFbqdgjoZz3FP7zrtofGwnRS4E6MCzmmD5Ni1Mxj");
const TOKEN_A_MINT: Pubkey = MINT_ADDRESS; // pubkey!("oreoU2P8bN6jkk3jbaiVxYnG1dCXcYxwhwyK9jSybcp");
const TOKEN_B_MINT: Pubkey = SOL_MINT; //pubkey!("So11111111111111111111111111111111111111112");
const A_VAULT: Pubkey = pubkey!("3s6ki6dQSM8FuqWiPsnGkgVsAEo8BTAfUR1Vvt1TPiJN");
const B_VAULT: Pubkey = pubkey!("FERjPVNEa7Udq8CEv68h6tPL46Tq7ieE49HrE2wea3XT");
const A_TOKEN_VAULT: Pubkey = pubkey!("BtJuiRG44vew5nYBVeUhuBawPTZLyYYxdzTYzerkfnto");
const B_TOKEN_VAULT: Pubkey = pubkey!("HZeLxbZ9uHtSpwZC3LBr4Nubd14iHwz7bRSghRZf5VCG");
const A_VAULT_LP_MINT: Pubkey = pubkey!("6Av9sdKvnjwoDHVnhEiz6JEq8e6SGzmhCsCncT2WJ7nN");
const B_VAULT_LP_MINT: Pubkey = pubkey!("FZN7QZ8ZUUAxMPfxYEYkH3cXUASzH8EqA6B4tyCL8f1j");
const A_VAULT_LP: Pubkey = pubkey!("2k7V1NtM1krwh1sdt5wWqBRcvNQ5jzxj3J2rV78zdTsL");
const B_VAULT_LP: Pubkey = pubkey!("CFATQFgkKXJyU3MdCNvQqN79qorNSMJFF8jrF66a7r6i");
const PROTOCOL_TOKEN_FEE: Pubkey = pubkey!("6kzYo2LMo2q2bkLAD8ienoG5NC1MkNXNTfm8sdyHuX3h");

// let [signer_info, config_info, mint_info, treasury_info, treasury_ore_info, treasury_sol_info, token_program] =

// let [pool, user_source_token, user_destination_token, a_vault, b_vault, a_token_vault, b_token_vault, a_vault_lp_mint, b_vault_lp_mint, a_vault_lp, b_vault_lp, protocol_token_fee, user_key, vault_program, token_program] =

pub fn bury(signer: Pubkey, min_amount_out: u64) -> Instruction {
    let board_address = board_pda().0;
    let config_address = config_pda().0;
    let mint_address = MINT_ADDRESS;
    let treasury_address = TREASURY_ADDRESS;
    let treasury_ore_address = get_associated_token_address(&treasury_address, &TOKEN_A_MINT);
    let treasury_sol_address = get_associated_token_address(&treasury_address, &TOKEN_B_MINT);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            // Ore accounts
            AccountMeta::new(signer, true),
            AccountMeta::new(board_address, false),
            AccountMeta::new_readonly(config_address, false),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new(treasury_ore_address, false),
            AccountMeta::new(treasury_sol_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(crate::ID, false),
            AccountMeta::new_readonly(meteora_pools_sdk::programs::AMM_ID, false),
            // Meteora accounts
            AccountMeta::new(POOL_ADDRESS, false),
            AccountMeta::new(treasury_sol_address, false),
            AccountMeta::new(treasury_ore_address, false),
            AccountMeta::new(A_VAULT, false),
            AccountMeta::new(B_VAULT, false),
            AccountMeta::new(A_TOKEN_VAULT, false),
            AccountMeta::new(B_TOKEN_VAULT, false),
            AccountMeta::new(A_VAULT_LP_MINT, false),
            AccountMeta::new(B_VAULT_LP_MINT, false),
            AccountMeta::new(A_VAULT_LP, false),
            AccountMeta::new(B_VAULT_LP, false),
            AccountMeta::new(PROTOCOL_TOKEN_FEE, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new_readonly(meteora_vault_sdk::programs::VAULT_ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: Bury {
            min_amount_out: min_amount_out.to_le_bytes(),
        }
        .to_bytes(),
    }
}

pub fn wrap(signer: Pubkey) -> Instruction {
    let config_address = config_pda().0;
    let treasury_address = TREASURY_ADDRESS;
    let treasury_sol_address = get_associated_token_address(&treasury_address, &TOKEN_B_MINT);
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

// let [signer_info, mint_info, seeker_info, stake_info, token_account_info, system_program] =

pub fn claim_seeker(signer: Pubkey, mint: Pubkey) -> Instruction {
    let seeker_address = seeker_pda(mint).0;
    let stake_address = stake_pda(signer).0;
    let token_account_address =
        get_associated_token_address_with_program_id(&signer, &mint, &spl_token_2022::ID);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(seeker_address, false),
            AccountMeta::new(stake_address, false),
            AccountMeta::new(token_account_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: ClaimSeeker {}.to_bytes(),
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

pub fn set_is_new_rng_enabled(signer: Pubkey, is_new_rng_enabled: bool) -> Instruction {
    let config_address = config_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(config_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: SetIsNewRngEnabled {
            is_new_rng_enabled: is_new_rng_enabled as u8,
        }
        .to_bytes(),
    }
}
