use steel::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct MineEvent {
    pub balance: u64,
    pub difficulty: u64,
    pub last_hash_at: i64,
    pub timing: i64,
    pub reward: u64,
    pub boost_1: u64,
    pub boost_2: u64,
    pub boost_3: u64,
}

event!(MineEvent);
