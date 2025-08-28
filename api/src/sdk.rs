use spl_associated_token_account::get_associated_token_address;
use steel::*;

use crate::{
    consts::{BOOST_RESERVE_TOKEN, MARKET, MINT_ADDRESS, TREASURY_ADDRESS},
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

pub fn initialize(signer: Pubkey) -> Instruction {
    let config_address = config_pda().0;
    let market_address = market_pda().0;
    let mint_address = MINT_ADDRESS;
    let treasury_address = TREASURY_ADDRESS;
    let treasury_tokens_address = treasury_tokens_address();
    let vault_address = vault_address();
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(config_address, false),
            AccountMeta::new(market_address, false),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new(treasury_tokens_address, false),
            AccountMeta::new(vault_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        ],
        data: Initialize {}.to_bytes(),
    }
}

pub fn mine(signer: Pubkey, id: u64, nonce: u64) -> Instruction {
    let block_adddress = block_pda(id).0;
    let miner_address = miner_pda(signer).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(miner_address, false),
        ],
        data: Mine {
            nonce: nonce.to_le_bytes(),
        }
        .to_bytes(),
    }
}

pub fn open(signer: Pubkey, id: u64) -> Instruction {
    let block_adddress = block_pda(id).0;
    let market_address = market_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(market_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: Open {
            id: id.to_le_bytes(),
        }
        .to_bytes(),
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

pub fn close(signer: Pubkey, opener: Pubkey, winner: Pubkey, id: u64) -> Instruction {
    let block_adddress = block_pda(id).0;
    let market_address = market_pda().0;
    let miner_address = miner_pda(winner).0;
    let miner_tokens_address = get_associated_token_address(&miner_address, &MINT_ADDRESS);
    let mint_address = MINT_ADDRESS;
    let treasury_address = TREASURY_ADDRESS;
    let treasury_tokens_address = treasury_tokens_address();
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(miner_address, false),
            AccountMeta::new_readonly(market_address, false),
            AccountMeta::new(miner_tokens_address, false),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(opener, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new(treasury_tokens_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        ],
        data: Close {}.to_bytes(),
    }
}

// let [signer_info, block_prev_info, block_next_info, config_info, market_info, mint_info, reserve_tokens_info, treasury_info, treasury_tokens_info, vault_info, system_program, token_program, ore_program, slot_hashes_sysvar] =

pub fn reset(signer: Pubkey, id: u64) -> Instruction {
    let block_prev_adddress = block_pda(id).0;
    let block_next_adddress = block_pda(id + 1).0;
    let config_address = config_pda().0;
    let market_address = market_pda().0;
    let mint_address = MINT_ADDRESS;
    let treasury_address = TREASURY_ADDRESS;
    let treasury_tokens_address = treasury_tokens_address();
    let vault_address = vault_address();
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_prev_adddress, false),
            AccountMeta::new(block_next_adddress, false),
            AccountMeta::new_readonly(config_address, false),
            AccountMeta::new(market_address, false),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(BOOST_RESERVE_TOKEN, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new(treasury_tokens_address, false),
            AccountMeta::new(vault_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(crate::ID, false),
            AccountMeta::new_readonly(sysvar::slot_hashes::ID, false),
        ],
        data: Reset {}.to_bytes(),
    }
}
// let [signer_info, block_info, config_info, fee_collector_info, market_info, miner_info, mint_info, tokens_info, vault_info, system_program, token_program, associated_token_program, ore_program] =

pub fn swap(
    signer: Pubkey,
    id: u64,
    fee_collector: Pubkey,
    amount: u64,
    direction: SwapDirection,
    precision: SwapPrecision,
    seed: [u8; 32],
) -> Instruction {
    let block_adddress = block_pda(id).0;
    let config_address = config_pda().0;
    let market_address = market_pda().0;
    let miner_address = miner_pda(signer).0;
    let tokens_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    let vault_address = vault_address();
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(block_adddress, false),
            AccountMeta::new(config_address, false),
            AccountMeta::new(fee_collector, false),
            AccountMeta::new(market_address, false),
            AccountMeta::new(miner_address, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(tokens_address, false),
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
            seed: seed,
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

pub fn set_block_duration(signer: Pubkey, block_duration: u64) -> Instruction {
    let config_address = config_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(config_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: SetBlockDuration {
            block_duration: block_duration.to_le_bytes(),
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

pub fn set_sniper_fee_duration(signer: Pubkey, sniper_fee_duration: u64) -> Instruction {
    let config_address = config_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(config_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: SetSniperFeeDuration {
            sniper_fee_duration: sniper_fee_duration.to_le_bytes(),
        }
        .to_bytes(),
    }
}

pub fn migrate_miner_account(signer: Pubkey, miner: Pubkey) -> Instruction {
    let config_address = config_pda().0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(config_address, false),
            AccountMeta::new(miner, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: MigrateMinerAccount {
            authority: miner.to_bytes(),
        }
        .to_bytes(),
    }
}
