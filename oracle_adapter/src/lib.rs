#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, Symbol
};

const LEDGERS_PER_DAY: u32 = 17_280;
const TTL_THRESHOLD: u32 = LEDGERS_PER_DAY * 5;
const TTL_EXTEND_TO: u32 = LEDGERS_PER_DAY * 30;

#[contracttype]
pub enum DataKey {
    Admin,
    Price(Symbol), // e.g., XLM, USDC
}

#[contract]
pub struct OracleAdapterContract;

#[contractimpl]
impl OracleAdapterContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialised");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
    }

    /// Admin function to push a mock price for an asset (e.g., testnet RedStone fallback)
    /// Price is expected to be scaled by 1e7 (10,000,000 = )
    pub fn set_price(env: Env, admin: Address, asset: Symbol, price: i128) {
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Unauthorised");
        }
        admin.require_auth();
        
        env.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
        env.storage().persistent().set(&DataKey::Price(asset.clone()), &price);
        env.storage().persistent().extend_ttl(&DataKey::Price(asset), TTL_THRESHOLD, TTL_EXTEND_TO);
    }

    /// Get the USD price of an asset, scaled by 1e7.
    /// Panic if price feed is unavailable.
    pub fn get_price(env: Env, asset: Symbol) -> i128 {
        let price = env.storage().persistent().get(&DataKey::Price(asset.clone()));
        if let Some(p) = price {
            env.storage().persistent().extend_ttl(&DataKey::Price(asset), TTL_THRESHOLD, TTL_EXTEND_TO);
            p
        } else {
            // Default fallback for testnet if not set
            if asset == Symbol::new(&env, "USDC") {
                10_000_000 // 
            } else if asset == Symbol::new(&env, "XLM") {
                1_000_000 // .10
            } else {
                panic!("Price not available for asset");
            }
        }
    }
}
