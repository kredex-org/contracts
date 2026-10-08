use soroban_sdk::Env;

const BASE_RATE_BPS: u32 = 200; // 2% base
const KINK_UTILIZATION: u32 = 8000; // 80%
const SLOPE1_BPS: u32 = 400; // 4% additional up to kink
const SLOPE2_BPS: u32 = 3000; // 30% additional after kink

pub fn calculate_dynamic_rate(_env: &Env, utilization: u32) -> u32 {
    if utilization <= KINK_UTILIZATION {
        // base + (utilization / 10000) * slope1
        let util_factor = (utilization as u64 * SLOPE1_BPS as u64) / 10000;
        BASE_RATE_BPS + (util_factor as u32)
    } else {
        // base + kink_slope + ((utilization - kink) / 10000) * slope2
        let util_factor1 = (KINK_UTILIZATION as u64 * SLOPE1_BPS as u64) / 10000;
        let excess_util = utilization - KINK_UTILIZATION;
        let util_factor2 = (excess_util as u64 * SLOPE2_BPS as u64) / 10000;
        
        BASE_RATE_BPS + (util_factor1 as u32) + (util_factor2 as u32)
    }
}
