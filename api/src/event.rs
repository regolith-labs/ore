use steel::*;

use crate::state::SwapDirection;

pub enum OreEvent {
    Reset = 0,
    Swap = 1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct ResetEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The authority of the swap.
    pub authority: Pubkey,

    /// The block that was opened for trading.
    pub block_id: u64,

    /// The start slot of the next block.
    pub start_slot: u64,

    /// The end slot of the next block.
    pub end_slot: u64,

    /// The timestamp of the event.
    pub ts: i64,
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

event!(ResetEvent);
event!(SwapEvent);
