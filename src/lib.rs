pub mod config;
pub mod contract;

use ethers::types::U256;

pub fn convert_to_u256_arr(input: [i32; 6]) -> [U256; 6] {
    let mut result = [U256::from(0); 6];

    for i in 0..6 {
        result[i] = U256::from(input[i]);
    }

    result
}
