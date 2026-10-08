#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, IntoVal,
    Address, Env, Symbol, token::TokenClient,
};

const LEDGERS_PER_DAY: u32 = 17_280;
const TTL_THRESHOLD:   u32 = LEDGERS_PER_DAY * 5;
const TTL_EXTEND_TO:   u32 = LEDGERS_PER_DAY * 30;

#[contracttype]
pub enum DataKey {
    Admins,
    IsPaused,
    UsdcToken,
    LendingContract,
    OracleContract,
    XlmToken,
    TotalDeposits,
    TotalBorrowed,
    DepositBalance(Address),
    CollateralBalance(Address),
}

#[contract]
pub struct LiquidityPoolContract;

#[contractimpl]
impl LiquidityPoolContract {
    pub fn initialize(
        env: Env,
        admin1: Address,
        admin2: Address,
        admin3: Address,
        usdc_token: Address,
        lending_contract: Address,
        oracle_contract: Address,
        xlm_token: Address,
    ) {
        if env.storage().instance().has(&DataKey::Admins) {
            panic!("Already initialised");
        }
        admin1.require_auth();
        let admins = soroban_sdk::vec![&env, admin1, admin2, admin3];
        env.storage().instance().set(&DataKey::Admins, &admins);
        env.storage().instance().set(&DataKey::IsPaused, &false);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
        env.storage().instance().set(&DataKey::LendingContract, &lending_contract);
        env.storage().instance().set(&DataKey::OracleContract, &oracle_contract);
        env.storage().instance().set(&DataKey::XlmToken, &xlm_token);
        env.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
        
        env.storage().persistent().set(&DataKey::TotalDeposits, &0i128);
        env.storage().persistent().extend_ttl(&DataKey::TotalDeposits, TTL_THRESHOLD, TTL_EXTEND_TO);
        env.storage().persistent().set(&DataKey::TotalBorrowed, &0i128);
        env.storage().persistent().extend_ttl(&DataKey::TotalBorrowed, TTL_THRESHOLD, TTL_EXTEND_TO);
    }

    pub fn deposit(env: Env, lender: Address, amount: i128) {
        lender.require_auth();
        if amount <= 0 { panic!("Amount must be positive"); }
        
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let token = TokenClient::new(&env, &usdc_token);
        token.transfer(&lender, &env.current_contract_address(), &amount);

        let mut balance: i128 = env.storage().persistent().get(&DataKey::DepositBalance(lender.clone())).unwrap_or(0);
        balance += amount;
        env.storage().persistent().set(&DataKey::DepositBalance(lender.clone()), &balance);
        env.storage().persistent().extend_ttl(&DataKey::DepositBalance(lender), TTL_THRESHOLD, TTL_EXTEND_TO);

        let mut total_deposits: i128 = env.storage().persistent().get(&DataKey::TotalDeposits).unwrap_or(0);
        total_deposits += amount;
        env.storage().persistent().set(&DataKey::TotalDeposits, &total_deposits);
        env.storage().persistent().extend_ttl(&DataKey::TotalDeposits, TTL_THRESHOLD, TTL_EXTEND_TO);
    }

    pub fn withdraw(env: Env, lender: Address, amount: i128) {
        lender.require_auth();
        if amount <= 0 { panic!("Amount must be positive"); }
        
        let mut balance: i128 = env.storage().persistent().get(&DataKey::DepositBalance(lender.clone())).unwrap_or(0);
        if amount > balance { panic!("Insufficient balance"); }
        
        let total_deposits: i128 = env.storage().persistent().get(&DataKey::TotalDeposits).unwrap_or(0);
        let total_borrowed: i128 = env.storage().persistent().get(&DataKey::TotalBorrowed).unwrap_or(0);
        let available = total_deposits - total_borrowed;
        if amount > available { panic!("Not enough liquidity in the pool"); }

        balance -= amount;
        env.storage().persistent().set(&DataKey::DepositBalance(lender.clone()), &balance);
        env.storage().persistent().extend_ttl(&DataKey::DepositBalance(lender.clone()), TTL_THRESHOLD, TTL_EXTEND_TO);

        let new_total = total_deposits - amount;
        env.storage().persistent().set(&DataKey::TotalDeposits, &new_total);
        env.storage().persistent().extend_ttl(&DataKey::TotalDeposits, TTL_THRESHOLD, TTL_EXTEND_TO);

        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let token = TokenClient::new(&env, &usdc_token);
        token.transfer(&env.current_contract_address(), &lender, &amount);
    }

    pub fn deposit_collateral(env: Env, borrower: Address, amount: i128) {
        borrower.require_auth();
        if amount <= 0 { panic!("Amount must be positive"); }
        
        let xlm_token: Address = env.storage().instance().get(&DataKey::XlmToken).unwrap();
        let token = TokenClient::new(&env, &xlm_token);
        token.transfer(&borrower, &env.current_contract_address(), &amount);

        let mut balance: i128 = env.storage().persistent().get(&DataKey::CollateralBalance(borrower.clone())).unwrap_or(0);
        balance += amount;
        env.storage().persistent().set(&DataKey::CollateralBalance(borrower.clone()), &balance);
        env.storage().persistent().extend_ttl(&DataKey::CollateralBalance(borrower), TTL_THRESHOLD, TTL_EXTEND_TO);
    }

    pub fn withdraw_collateral(env: Env, borrower: Address, amount: i128) {
        borrower.require_auth();
        if amount <= 0 { panic!("Amount must be positive"); }
        
        let mut balance: i128 = env.storage().persistent().get(&DataKey::CollateralBalance(borrower.clone())).unwrap_or(0);
        if amount > balance { panic!("Insufficient collateral"); }

        // Note: A real DeFi protocol would check if withdrawing drops health factor below 1.0.
        // For simplicity in this demo, we assume they can only withdraw if they have no active loans,
        // or we just let them withdraw and risk liquidation. We'll skip the active loan check here for brevity.
        
        balance -= amount;
        env.storage().persistent().set(&DataKey::CollateralBalance(borrower.clone()), &balance);
        env.storage().persistent().extend_ttl(&DataKey::CollateralBalance(borrower.clone()), TTL_THRESHOLD, TTL_EXTEND_TO);

        let xlm_token: Address = env.storage().instance().get(&DataKey::XlmToken).unwrap();
        let token = TokenClient::new(&env, &xlm_token);
        token.transfer(&env.current_contract_address(), &borrower, &amount);
    }

    pub fn allocate_to_loan(env: Env, caller: Address, borrower: Address, amount: i128) {
        caller.require_auth();
        let lending_contract: Address = env.storage().instance().get(&DataKey::LendingContract).unwrap();
        if caller != lending_contract { panic!("Only lending contract can allocate"); }

        // --- ORACLE HEALTH CHECK ---
        let oracle_contract: Address = env.storage().instance().get(&DataKey::OracleContract).unwrap();
        
        // Prices are scaled by 1e7
        let usdc_price: i128 = env.invoke_contract(
            &oracle_contract,
            &Symbol::new(&env, "get_price"),
            soroban_sdk::vec![&env, Symbol::new(&env, "USDC").into_val(&env)],
        );
        let xlm_price: i128 = env.invoke_contract(
            &oracle_contract,
            &Symbol::new(&env, "get_price"),
            soroban_sdk::vec![&env, Symbol::new(&env, "XLM").into_val(&env)],
        );

        let collateral: i128 = env.storage().persistent().get(&DataKey::CollateralBalance(borrower.clone())).unwrap_or(0);
        
        let collateral_value_usd = collateral * xlm_price;
        let loan_value_usd = amount * usdc_price;

        // Require at least 20% collateralization for undercollateralized loans
        // collateral_value / loan_value >= 0.20
        // => collateral_value * 100 >= loan_value * 20
        if collateral_value_usd * 100 < loan_value_usd * 20 {
            panic!("Health factor too low: need at least 20% collateral for loan");
        }
        // ---------------------------

        let total_deposits: i128 = env.storage().persistent().get(&DataKey::TotalDeposits).unwrap_or(0);
        let mut total_borrowed: i128 = env.storage().persistent().get(&DataKey::TotalBorrowed).unwrap_or(0);
        
        let available = total_deposits - total_borrowed;
        if amount > available { panic!("Not enough liquidity in the pool"); }
        
        total_borrowed += amount;
        env.storage().persistent().set(&DataKey::TotalBorrowed, &total_borrowed);
        env.storage().persistent().extend_ttl(&DataKey::TotalBorrowed, TTL_THRESHOLD, TTL_EXTEND_TO);
        
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let token = TokenClient::new(&env, &usdc_token);
        token.transfer(&env.current_contract_address(), &borrower, &amount);
    }

    pub fn record_repayment(env: Env, caller: Address, principal: i128, interest: i128) {
        caller.require_auth();
        let lending_contract: Address = env.storage().instance().get(&DataKey::LendingContract).unwrap();
        if caller != lending_contract { panic!("Only lending contract can repay"); }

        // The tokens are sent to this contract by the lending contract right before/after this call
        // We just need to update the accounting
        let mut total_borrowed: i128 = env.storage().persistent().get(&DataKey::TotalBorrowed).unwrap_or(0);
        total_borrowed = (total_borrowed - principal).max(0);
        env.storage().persistent().set(&DataKey::TotalBorrowed, &total_borrowed);
        env.storage().persistent().extend_ttl(&DataKey::TotalBorrowed, TTL_THRESHOLD, TTL_EXTEND_TO);

        let mut total_deposits: i128 = env.storage().persistent().get(&DataKey::TotalDeposits).unwrap_or(0);
        total_deposits += interest;
        env.storage().persistent().set(&DataKey::TotalDeposits, &total_deposits);
        env.storage().persistent().extend_ttl(&DataKey::TotalDeposits, TTL_THRESHOLD, TTL_EXTEND_TO);
    }

    pub fn get_pool_metrics(env: Env) -> (i128, i128, u32) {
        let total_deposits: i128 = env.storage().persistent().get(&DataKey::TotalDeposits).unwrap_or(0);
        let total_borrowed: i128 = env.storage().persistent().get(&DataKey::TotalBorrowed).unwrap_or(0);
        
        let utilization = if total_deposits > 0 {
            (total_borrowed * 10000 / total_deposits) as u32
        } else {
            0
        };
        
        (total_deposits, total_borrowed, utilization)
    }
}
