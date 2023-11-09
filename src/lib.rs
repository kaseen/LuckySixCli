pub mod config;
pub mod contract;

use ethers::types::U256;

#[derive(Debug, PartialEq, Eq)]
pub enum LotteryState {
    READY,
    STARTED,
    CALCULATING,
    DRAWING,
    CLOSED,
}

impl From<u8> for LotteryState {
    fn from(num: u8) -> Self {
        match num {
            0 => LotteryState::READY,
            1 => LotteryState::STARTED,
            2 => LotteryState::CALCULATING,
            3 => LotteryState::DRAWING,
            4 => LotteryState::CLOSED,
            _ => panic!("Invalid lottery state"),
        }
    }
}

pub fn convert_to_u256_arr(input: [i32; 6]) -> [U256; 6] {
    let mut result = [U256::from(0); 6];

    for i in 0..6 {
        result[i] = U256::from(input[i]);
    }

    result
}
