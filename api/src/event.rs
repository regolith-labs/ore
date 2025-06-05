use steel::*;

use crate::state::SwapDirection;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct SwapEvent {
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
}

impl SwapEvent {
    pub fn direction(&self) -> SwapDirection {
        SwapDirection::try_from(self.direction as u8).unwrap()
    }
}

event!(SwapEvent);
