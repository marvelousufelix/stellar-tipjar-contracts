#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, token, Address,
    Env,
};

#[cfg(test)]
extern crate std;

/// Storage layout for persistent contract data.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Token contract address used for all tips.
    Token,
    /// Creator's currently withdrawable balance held by this contract.
    CreatorBalance(Address),
    /// Historical total tips ever received by creator.
    CreatorTotal(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TipJarError {
    AlreadyInitialized = 1,
    TokenNotInitialized = 2,
    InvalidAmount = 3,
    NothingToWithdraw = 4,
}

#[contract]
pub struct TipJarContract;

#[contractimpl]
impl TipJarContract {
    /// One-time setup to choose the token contract accepted for tips.
    pub fn init(env: Env, token: Address) {
        if env.storage().instance().has(&DataKey::Token) {
            panic_with_error!(&env, TipJarError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Token, &token);
    }

    /// Moves `amount` tokens from `sender` into contract escrow for `creator`.
    ///
    /// The sender must authorize this call and have enough token balance.
    pub fn tip(env: Env, sender: Address, creator: Address, amount: i128) {
        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }

        sender.require_auth();

        let token_id = Self::read_token(&env);
        let token_client = token::Client::new(&env, &token_id);
        let contract_address = env.current_contract_address();

        // Transfer tokens into contract escrow first so creators can withdraw later.
        token_client.transfer(&sender, &contract_address, &amount);

        let creator_balance_key = DataKey::CreatorBalance(creator.clone());
        let creator_total_key = DataKey::CreatorTotal(creator.clone());

        let current_balance: i128 = env
            .storage()
            .persistent()
            .get(&creator_balance_key)
            .unwrap_or(0);
        let current_total: i128 = env
            .storage()
            .persistent()
            .get(&creator_total_key)
            .unwrap_or(0);

        let next_balance = current_balance + amount;
        let next_total = current_total + amount;

        env.storage().persistent().set(&creator_balance_key, &next_balance);
        env.storage().persistent().set(&creator_total_key, &next_total);

        // Event topics: ("tip", creator). Event data: (sender, amount).
        env.events()
            .publish((symbol_short!("tip"), creator), (sender, amount));
    }

    /// Returns total historical tips for a creator.
    pub fn get_total_tips(env: Env, creator: Address) -> i128 {
        let key = DataKey::CreatorTotal(creator);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    /// Returns currently withdrawable escrowed tips for a creator.
    pub fn get_withdrawable_balance(env: Env, creator: Address) -> i128 {
        let key = DataKey::CreatorBalance(creator);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    /// Allows creator to withdraw their accumulated escrowed tips.
    pub fn withdraw(env: Env, creator: Address) {
        creator.require_auth();

        let key = DataKey::CreatorBalance(creator.clone());
        let amount: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        if amount <= 0 {
            panic_with_error!(&env, TipJarError::NothingToWithdraw);
        }

        let token_id = Self::read_token(&env);
        let token_client = token::Client::new(&env, &token_id);
        let contract_address = env.current_contract_address();

        token_client.transfer(&contract_address, &creator, &amount);
        env.storage().persistent().set(&key, &0i128);

        env.events()
            .publish((symbol_short!("withdraw"), creator), amount);
    }

    fn read_token(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Token)
            .unwrap_or_else(|| panic_with_error!(env, TipJarError::TokenNotInitialized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, token, Address, Env};

    fn setup() -> (Env, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let token_admin = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract(token_admin.clone());

        let contract_id = env.register_contract(None, TipJarContract);
        let tipjar_client = TipJarContractClient::new(&env, &contract_id);
        tipjar_client.init(&token_id);

        (env, contract_id, token_id)
    }

    #[test]
    fn test_tipping_functionality() {
        let (env, contract_id, token_id) = setup();
        let tipjar_client = TipJarContractClient::new(&env, &contract_id);
        let token_client = token::Client::new(&env, &token_id);
        let token_admin_client = token::StellarAssetClient::new(&env, &token_id);
        let sender = Address::generate(&env);
        let creator = Address::generate(&env);

        token_admin_client.mint(&sender, &1_000);
        tipjar_client.tip(&sender, &creator, &250);

        assert_eq!(token_client.balance(&sender), 750);
        assert_eq!(token_client.balance(&contract_id), 250);
        assert_eq!(tipjar_client.get_total_tips(&creator), 250);
    }

    #[test]
    fn test_balance_tracking_and_withdraw() {
        let (env, contract_id, token_id) = setup();
        let tipjar_client = TipJarContractClient::new(&env, &contract_id);
        let token_client = token::Client::new(&env, &token_id);
        let token_admin_client = token::StellarAssetClient::new(&env, &token_id);
        let sender_a = Address::generate(&env);
        let sender_b = Address::generate(&env);
        let creator = Address::generate(&env);

        token_admin_client.mint(&sender_a, &1_000);
        token_admin_client.mint(&sender_b, &1_000);

        tipjar_client.tip(&sender_a, &creator, &100);
        tipjar_client.tip(&sender_b, &creator, &300);

        assert_eq!(tipjar_client.get_total_tips(&creator), 400);
        assert_eq!(tipjar_client.get_withdrawable_balance(&creator), 400);
        assert_eq!(token_client.balance(&contract_id), 400);

        tipjar_client.withdraw(&creator);

        assert_eq!(tipjar_client.get_withdrawable_balance(&creator), 0);
        assert_eq!(token_client.balance(&creator), 400);
        assert_eq!(token_client.balance(&contract_id), 0);
    }

    #[test]
    #[should_panic]
    fn test_invalid_tip_amount() {
        let (env, contract_id, token_id) = setup();
        let tipjar_client = TipJarContractClient::new(&env, &contract_id);
        let token_client = token::Client::new(&env, &token_id);
        let token_admin_client = token::StellarAssetClient::new(&env, &token_id);
        let sender = Address::generate(&env);
        let creator = Address::generate(&env);

        token_admin_client.mint(&sender, &100);

        // Zero tips are rejected to prevent accidental or abusive calls.
        tipjar_client.tip(&sender, &creator, &0);

        // Keep these in scope to avoid accidental unused-variable edits in refactors.
        let _ = env;
        let _ = token_client;
    }
}
