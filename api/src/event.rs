use steel::*;

pub enum OreEvent {
    Reset = 0,
    Mine = 2,
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
pub struct MineEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The authority of the mine.
    pub authority: Pubkey,

    /// The block that was mined.
    pub block_id: u64,

    /// The nonce that was mined.
    pub nonce: u64,

    /// The total hashpower the miner had.
    pub hashpower: u64,

    /// Whether or not the miner is the new best.
    pub is_best: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

event!(ResetEvent);
event!(MineEvent);
