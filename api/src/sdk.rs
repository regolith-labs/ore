use algonaut_core::Address;
use algonaut_transaction::{
    transaction::{ApplicationCallTransaction, Transaction},
    tx_group::TxGroup,
};
use serde::{Deserialize, Serialize};

use crate::{
    consts::*,
    instruction::*,
    state::*,
    APP_ID,
};

/// Application call arguments for fPOW transactions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppCallArgs {
    pub method: String,
    pub args: Vec<Vec<u8>>,
}

/// Build an Algorand application call transaction
pub fn build_app_call(
    sender: Address,
    method: FpowInstruction,
    args: Vec<Vec<u8>>,
    boxes: Vec<(u64, Vec<u8>)>,
    foreign_assets: Vec<u64>,
    foreign_apps: Vec<u64>,
    accounts: Vec<Address>,
) -> ApplicationCallTransaction {
    ApplicationCallTransaction {
        sender,
        app_id: APP_ID,
        on_complete: algonaut_transaction::transaction::OnComplete::NoOp,
        approval_program: None,
        clear_program: None,
        global_state_schema: None,
        local_state_schema: None,
        app_arguments: Some(build_method_args(method, args)),
        accounts: if accounts.is_empty() { None } else { Some(accounts) },
        foreign_apps: if foreign_apps.is_empty() { None } else { Some(foreign_apps) },
        foreign_assets: if foreign_assets.is_empty() { None } else { Some(foreign_assets) },
        extra_pages: None,
        boxes: if boxes.is_empty() { None } else { Some(boxes.into_iter().map(|(app_id, name)| (app_id, name)).collect()) },
    }
}

/// Build method arguments for ABI-compliant application call
fn build_method_args(method: FpowInstruction, args: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let mut result = vec![method_selector_bytes(method)];
    result.extend(args);
    result
}

/// Get the 4-byte method selector for an instruction
fn method_selector_bytes(method: FpowInstruction) -> Vec<u8> {
    use sha2::{Sha512_256, Digest};
    let selector = method.method_selector();
    let hash = Sha512_256::digest(selector.as_bytes());
    hash[0..4].to_vec()
}

/// Build automate transaction
pub fn automate(
    sender: Address,
    amount: u64,
    deposit: u64,
    executor: Address,
    fee: u64,
    mask: u64,
    strategy: u8,
    reload: bool,
) -> ApplicationCallTransaction {
    let args = AutomateArgs {
        amount,
        deposit,
        fee,
        mask,
        strategy,
        reload: if reload { 1 } else { 0 },
    };

    let sender_bytes: [u8; 32] = sender.0;
    let miner_box = miner_box_name(&sender_bytes);
    let automation_box = automation_box_name(&sender_bytes);

    build_app_call(
        sender,
        FpowInstruction::Automate,
        vec![
            amount.to_be_bytes().to_vec(),
            deposit.to_be_bytes().to_vec(),
            fee.to_be_bytes().to_vec(),
            mask.to_be_bytes().to_vec(),
            vec![strategy],
            (if reload { 1u64 } else { 0u64 }).to_be_bytes().to_vec(),
        ],
        vec![
            (APP_ID, miner_box),
            (APP_ID, automation_box),
        ],
        vec![],
        vec![],
        vec![executor],
    )
}

/// Build claim ALGO transaction
pub fn claim_algo(sender: Address) -> ApplicationCallTransaction {
    let sender_bytes: [u8; 32] = sender.0;
    let miner_box = miner_box_name(&sender_bytes);

    build_app_call(
        sender,
        FpowInstruction::ClaimALGO,
        vec![],
        vec![(APP_ID, miner_box)],
        vec![],
        vec![],
        vec![],
    )
}

/// Build claim fPOW transaction
pub fn claim_fpow(sender: Address) -> ApplicationCallTransaction {
    let sender_bytes: [u8; 32] = sender.0;
    let miner_box = miner_box_name(&sender_bytes);
    let treasury_box = treasury_box_name();

    build_app_call(
        sender,
        FpowInstruction::ClaimFPOW,
        vec![],
        vec![
            (APP_ID, miner_box),
            (APP_ID, treasury_box),
        ],
        vec![FPOW_ASA_ID],
        vec![],
        vec![],
    )
}

/// Build deploy transaction
pub fn deploy(
    sender: Address,
    authority: Address,
    amount: u64,
    round_id: u64,
    squares: [bool; 25],
) -> ApplicationCallTransaction {
    // Convert array of 25 booleans into a 32-bit mask
    let mut mask: u32 = 0;
    for (i, &square) in squares.iter().enumerate() {
        if square {
            mask |= 1 << i;
        }
    }

    let authority_bytes: [u8; 32] = authority.0;
    let miner_box = miner_box_name(&authority_bytes);
    let automation_box = automation_box_name(&authority_bytes);
    let board_box = board_box_name();
    let round_box = round_box_name(round_id);
    let config_box = config_box_name();

    build_app_call(
        sender,
        FpowInstruction::Deploy,
        vec![
            amount.to_be_bytes().to_vec(),
            mask.to_be_bytes().to_vec(),
        ],
        vec![
            (APP_ID, miner_box),
            (APP_ID, automation_box),
            (APP_ID, board_box),
            (APP_ID, round_box),
            (APP_ID, config_box),
        ],
        vec![],
        vec![],
        vec![authority],
    )
}

/// Build reset transaction
pub fn reset(
    sender: Address,
    fee_collector: Address,
    round_id: u64,
    top_miner: Address,
) -> ApplicationCallTransaction {
    let top_miner_bytes: [u8; 32] = top_miner.0;
    let miner_box = miner_box_name(&top_miner_bytes);
    let board_box = board_box_name();
    let config_box = config_box_name();
    let round_box = round_box_name(round_id);
    let next_round_box = round_box_name(round_id + 1);
    let treasury_box = treasury_box_name();

    build_app_call(
        sender,
        FpowInstruction::Reset,
        vec![],
        vec![
            (APP_ID, board_box),
            (APP_ID, config_box),
            (APP_ID, round_box),
            (APP_ID, next_round_box),
            (APP_ID, miner_box),
            (APP_ID, treasury_box),
        ],
        vec![FPOW_ASA_ID],
        vec![],
        vec![fee_collector, top_miner],
    )
}

/// Build close transaction
pub fn close(sender: Address, round_id: u64, rent_payer: Address) -> ApplicationCallTransaction {
    let board_box = board_box_name();
    let round_box = round_box_name(round_id);
    let treasury_box = treasury_box_name();

    build_app_call(
        sender,
        FpowInstruction::Close,
        vec![],
        vec![
            (APP_ID, board_box),
            (APP_ID, round_box),
            (APP_ID, treasury_box),
        ],
        vec![],
        vec![],
        vec![rent_payer],
    )
}

/// Build checkpoint transaction
pub fn checkpoint(sender: Address, authority: Address, round_id: u64) -> ApplicationCallTransaction {
    let authority_bytes: [u8; 32] = authority.0;
    let miner_box = miner_box_name(&authority_bytes);
    let board_box = board_box_name();
    let round_box = round_box_name(round_id);
    let treasury_box = treasury_box_name();

    build_app_call(
        sender,
        FpowInstruction::Checkpoint,
        vec![],
        vec![
            (APP_ID, board_box),
            (APP_ID, miner_box),
            (APP_ID, round_box),
            (APP_ID, treasury_box),
        ],
        vec![],
        vec![],
        vec![authority],
    )
}

/// Build set_admin transaction
pub fn set_admin(sender: Address, admin: Address) -> ApplicationCallTransaction {
    let config_box = config_box_name();

    build_app_call(
        sender,
        FpowInstruction::SetAdmin,
        vec![admin.0.to_vec()],
        vec![(APP_ID, config_box)],
        vec![],
        vec![],
        vec![admin],
    )
}

/// Build deposit transaction
pub fn deposit(sender: Address, amount: u64, compound_fee: u64) -> ApplicationCallTransaction {
    let sender_bytes: [u8; 32] = sender.0;
    let stake_box = stake_box_name(&sender_bytes);
    let treasury_box = treasury_box_name();

    build_app_call(
        sender,
        FpowInstruction::Deposit,
        vec![
            amount.to_be_bytes().to_vec(),
            compound_fee.to_be_bytes().to_vec(),
        ],
        vec![
            (APP_ID, stake_box),
            (APP_ID, treasury_box),
        ],
        vec![FPOW_ASA_ID],
        vec![],
        vec![],
    )
}

/// Build withdraw transaction
pub fn withdraw(sender: Address, amount: u64) -> ApplicationCallTransaction {
    let sender_bytes: [u8; 32] = sender.0;
    let stake_box = stake_box_name(&sender_bytes);
    let treasury_box = treasury_box_name();

    build_app_call(
        sender,
        FpowInstruction::Withdraw,
        vec![amount.to_be_bytes().to_vec()],
        vec![
            (APP_ID, stake_box),
            (APP_ID, treasury_box),
        ],
        vec![FPOW_ASA_ID],
        vec![],
        vec![],
    )
}

/// Build reload_algo transaction
pub fn reload_algo(sender: Address, authority: Address) -> ApplicationCallTransaction {
    let authority_bytes: [u8; 32] = authority.0;
    let automation_box = automation_box_name(&authority_bytes);
    let miner_box = miner_box_name(&authority_bytes);

    build_app_call(
        sender,
        FpowInstruction::ReloadALGO,
        vec![],
        vec![
            (APP_ID, automation_box),
            (APP_ID, miner_box),
        ],
        vec![],
        vec![],
        vec![authority],
    )
}

/// Build claim_yield transaction
pub fn claim_yield(sender: Address, amount: u64) -> ApplicationCallTransaction {
    let sender_bytes: [u8; 32] = sender.0;
    let stake_box = stake_box_name(&sender_bytes);
    let treasury_box = treasury_box_name();

    build_app_call(
        sender,
        FpowInstruction::ClaimYield,
        vec![amount.to_be_bytes().to_vec()],
        vec![
            (APP_ID, stake_box),
            (APP_ID, treasury_box),
        ],
        vec![FPOW_ASA_ID],
        vec![],
        vec![],
    )
}

/// Build compound_yield transaction
pub fn compound_yield(sender: Address) -> ApplicationCallTransaction {
    let sender_bytes: [u8; 32] = sender.0;
    let stake_box = stake_box_name(&sender_bytes);
    let treasury_box = treasury_box_name();

    build_app_call(
        sender,
        FpowInstruction::CompoundYield,
        vec![],
        vec![
            (APP_ID, stake_box),
            (APP_ID, treasury_box),
        ],
        vec![FPOW_ASA_ID],
        vec![],
        vec![],
    )
}

/// Build bury transaction
pub fn bury(sender: Address, amount: u64) -> ApplicationCallTransaction {
    let board_box = board_box_name();
    let treasury_box = treasury_box_name();

    build_app_call(
        sender,
        FpowInstruction::Bury,
        vec![amount.to_be_bytes().to_vec()],
        vec![
            (APP_ID, board_box),
            (APP_ID, treasury_box),
        ],
        vec![FPOW_ASA_ID],
        vec![],
        vec![],
    )
}

/// Build wrap transaction
pub fn wrap(sender: Address, amount: u64) -> ApplicationCallTransaction {
    let config_box = config_box_name();
    let treasury_box = treasury_box_name();

    build_app_call(
        sender,
        FpowInstruction::Wrap,
        vec![amount.to_be_bytes().to_vec()],
        vec![
            (APP_ID, config_box),
            (APP_ID, treasury_box),
        ],
        vec![],
        vec![],
        vec![],
    )
}

/// Build liq transaction
pub fn liq(sender: Address, manager: Address) -> ApplicationCallTransaction {
    let board_box = board_box_name();
    let config_box = config_box_name();
    let treasury_box = treasury_box_name();

    build_app_call(
        sender,
        FpowInstruction::Liq,
        vec![],
        vec![
            (APP_ID, board_box),
            (APP_ID, config_box),
            (APP_ID, treasury_box),
        ],
        vec![],
        vec![],
        vec![manager],
    )
}

/// Build new_var transaction
pub fn new_var(
    sender: Address,
    provider: Address,
    id: u64,
    commit: [u8; 32],
    samples: u64,
) -> ApplicationCallTransaction {
    let board_box = board_box_name();
    let config_box = config_box_name();

    build_app_call(
        sender,
        FpowInstruction::NewVar,
        vec![
            id.to_be_bytes().to_vec(),
            commit.to_vec(),
            samples.to_be_bytes().to_vec(),
        ],
        vec![
            (APP_ID, board_box),
            (APP_ID, config_box),
        ],
        vec![],
        vec![],
        vec![provider],
    )
}

/// Build log transaction (for debugging/events)
pub fn log(sender: Address, msg: &[u8]) -> ApplicationCallTransaction {
    build_app_call(
        sender,
        FpowInstruction::Log,
        vec![msg.to_vec()],
        vec![],
        vec![],
        vec![],
        vec![],
    )
}
