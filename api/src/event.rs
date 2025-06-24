use steel::*;

use crate::state::{RewardConfig, SwapDirection};

pub enum OreEvent {
    Commit = 0,
    Deposit = 1,
    Mine = 2,
    Open = 3,
    Reward = 4,
    Swap = 5,
    Uncommit = 6,
    Withdraw = 7,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct SwapEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The authority of the swap.
    pub authority: Pubkey,

    /// The block id.
    pub block_id: u64,

    /// Swap direction.
    pub direction: u64,

    /// Amount of base tokens to transfer.
    pub base_to_transfer: u64,

    /// Amount of quote tokens to transfer.
    pub quote_to_transfer: u64,

    /// Amount of base tokens swapped via virtual limit order.
    pub base_via_order: u64,

    /// Amount of quote tokens swapped via virtual limit order.
    pub quote_via_order: u64,

    /// Amount of base tokens swapped via curve.
    pub base_via_curve: u64,

    /// Amount of quote tokens swapped via curve.
    pub quote_via_curve: u64,

    /// Amount of quote tokens taken in fees.
    pub quote_fee: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

impl SwapEvent {
    pub fn direction(&self) -> SwapDirection {
        SwapDirection::try_from(self.direction as u8).unwrap()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct RewardEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The amount of ORE distributed as a reward.
    pub amount: u64,

    /// The authority who received the reward.
    pub authority: Pubkey,

    /// The block id.
    pub block_id: u64,

    /// The type of reward.
    pub rewards_type: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct MineEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The authority who mined.
    pub authority: Pubkey,

    /// The block id.
    pub block_id: u64,

    /// The amount of hashes deployed.
    pub deployed: u64,

    /// The total amount of hashes deployed in the block.
    pub total_deployed: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum RewardsType {
    Nugget = 0,
    Lode = 1,
    Motherlode = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct OpenEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The signer of the open transaction.
    pub signer: Pubkey,

    /// The id of the block.
    pub id: u64,

    /// The start slot of the block.
    pub start_slot: u64,

    /// The base liquidity in the market.
    pub liquidity_base: u64,

    /// The quote liquidity in the market.
    pub liquidity_quote: u64,

    /// The reward configuration.
    pub reward_config: RewardConfig,

    /// The timestamp of the event.
    pub ts: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct CloseEvent {
    /// The authority of the close transaction.
    pub authority: Pubkey,

    /// The id of the block.
    pub id: u64,

    /// The amount of base (hashpower) liquidity burned.
    pub burned_base: u64,

    /// The amount of quote (ORE) liquidity burned.
    pub burned_quote: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct CommitEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The authority of the commit transaction.
    pub authority: Pubkey,

    /// The id of the block.
    pub block_id: u64,

    /// The amount of hashpower committed.
    pub amount: u64,

    /// The total amount of hashpower committed to the block.
    pub block_commitment: u64,

    /// The total amount of hashpower this miner has committed to the block.
    pub permit_commitment: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct UncommitEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The authority of the commit transaction.
    pub authority: Pubkey,

    /// The id of the block.
    pub block_id: u64,

    /// The amount of hashpower committed.
    pub amount: u64,

    /// The total amount of hashpower committed to the block.
    pub block_commitment: u64,

    /// The total amount of hashpower this miner has committed to the block.
    pub permit_commitment: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct DepositEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The authority of the commit transaction.
    pub authority: Pubkey,

    /// The id of the block.
    pub block_id: u64,

    /// The amount of ORE collateral deposited.
    pub amount: u64,

    /// The total amount of ORE this user has deposited as collateral.
    pub collateral: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct WithdrawEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The authority of the commit transaction.
    pub authority: Pubkey,

    /// The id of the block.
    pub block_id: u64,

    /// The amount of ORE collateral withdrawn.
    pub amount: u64,

    /// The total amount of ORE this user has deposited as collateral.
    pub collateral: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

event!(SwapEvent);
event!(RewardEvent);
event!(OpenEvent);
event!(CommitEvent);
event!(DepositEvent);
event!(WithdrawEvent);
event!(UncommitEvent);
event!(MineEvent);
event!(CloseEvent);
