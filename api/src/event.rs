use steel::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct MineEvent {
    pub balance: u64,
    pub difficulty: u64,
    pub last_hash_at: i64,
    pub timing: i64,
    pub net_reward: u64,
    pub net_base_reward: u64,
    pub net_miner_boost_reward: u64,
    pub net_staker_boost_reward: u64,
}

event!(MineEvent);
