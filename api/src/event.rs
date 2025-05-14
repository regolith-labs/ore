use steel::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct BlockEvent {
    pub score: u64,
    pub block_reward: u64,
    pub boost_reward: u64,
    pub ts: u64,
}

event!(BlockEvent);
