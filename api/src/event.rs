use steel::*;

use crate::state::SwapDirection;

pub enum OreEvent {
    Mine = 1,
    Reward = 2,
    Swap = 3,
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

    /// Amount of base tokens in the market.
    pub base_liquidity: u64,

    /// Amount of quote tokens in the market.
    pub quote_liquidity: u64,

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

    /// The amount of hashpower remaining in the permit.
    pub remaining_commitment: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

event!(SwapEvent);
event!(RewardEvent);
event!(MineEvent);
