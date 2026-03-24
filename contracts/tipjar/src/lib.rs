#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, token,
    Address, Env,
};

#[cfg(test)]
extern crate std;

/// Storage layout for persistent contract data.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Token contract address whitelist state (bool).
    TokenWhitelist(Address),
    /// Creator's currently withdrawable balance held by this contract per token.
    CreatorBalance(Address, Address), // (creator, token)
    /// Historical total tips ever received by creator per token.
    CreatorTotal(Address, Address),   // (creator, token)
    /// Emergency pause state (bool).
    Paused,
    /// Contract administrator (Address).
    Admin,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TipJarError {
    AlreadyInitialized = 1,
    TokenNotWhitelisted = 2,
    InvalidAmount = 3,
    NothingToWithdraw = 4,
    Unauthorized = 5,
}

#[contract]
pub struct TipJarContract;

#[contractimpl]
impl TipJarContract {
    /// One-time setup to choose the administrator for the TipJar.
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, TipJarError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
    }

    /// Adds a token to the whitelist (Admin only).
    pub fn add_token(env: Env, admin: Address, token: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage()
            .instance()
            .set(&DataKey::TokenWhitelist(token), &true);
    }

    /// Removes a token from the whitelist (Admin only).
    pub fn remove_token(env: Env, admin: Address, token: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage()
            .instance()
            .set(&DataKey::TokenWhitelist(token), &false);
    }

    /// Moves `amount` tokens from `sender` into contract escrow for `creator`.
    ///
    /// The sender must authorize this call and have enough token balance.
    pub fn tip(env: Env, sender: Address, creator: Address, token: Address, amount: i128) {
        if Self::is_paused(&env) {
            panic!("Contract is paused");
        }
        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }

        if !Self::is_whitelisted(env.clone(), token.clone()) {
            panic_with_error!(&env, TipJarError::TokenNotWhitelisted);
        }

        sender.require_auth();

        let token_client = token::Client::new(&env, &token);
        let contract_address = env.current_contract_address();

        // Transfer tokens into contract escrow first so creators can withdraw later.
        token_client.transfer(&sender, &contract_address, &amount);

        let creator_balance_key = DataKey::CreatorBalance(creator.clone(), token.clone());
        let creator_total_key = DataKey::CreatorTotal(creator.clone(), token.clone());

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

        env.storage()
            .persistent()
            .set(&creator_balance_key, &next_balance);
        env.storage()
            .persistent()
            .set(&creator_total_key, &next_total);

        // Event topics: ("tip", creator, token). Event data: (sender, amount).
        env.events()
            .publish((symbol_short!("tip"), creator, token), (sender, amount));
    }

    /// Returns total historical tips for a creator in a specific token.
    pub fn get_total_tips(env: Env, creator: Address, token: Address) -> i128 {
        let key = DataKey::CreatorTotal(creator, token);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    /// Returns currently withdrawable escrowed tips for a creator in a specific token.
    pub fn get_withdrawable_balance(env: Env, creator: Address, token: Address) -> i128 {
        let key = DataKey::CreatorBalance(creator, token);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    /// Allows creator to withdraw their accumulated escrowed tips for a specific token.
    pub fn withdraw(env: Env, creator: Address, token: Address) {
        if Self::is_paused(&env) {
            panic!("Contract is paused");
        }
        creator.require_auth();

        let key = DataKey::CreatorBalance(creator.clone(), token.clone());
        let amount: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        if amount <= 0 {
            panic_with_error!(&env, TipJarError::NothingToWithdraw);
        }

        let token_client = token::Client::new(&env, &token);
        let contract_address = env.current_contract_address();

        token_client.transfer(&contract_address, &creator, &amount);
        env.storage().persistent().set(&key, &0i128);

        env.events()
            .publish((symbol_short!("withdraw"), creator, token), amount);
    }

    pub fn is_whitelisted(env: Env, token: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::TokenWhitelist(token))
            .unwrap_or(false)
    }

    /// Emergency pause to stop all state-changing activities (Admin only).
    pub fn pause(env: Env, admin: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap_or_else(|| panic!("Not initialized"));
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::Paused, &true);
    }

    /// Resume contract activities after an emergency pause (Admin only).
    pub fn unpause(env: Env, admin: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap_or_else(|| panic!("Not initialized"));
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::Paused, &false);
    }

    /// Internal helper to check if the contract is paused.
    fn is_paused(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, token, Address, Env};

    fn setup() -> (Env, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let token_admin = Address::generate(&env);
        let token_id_1 = env
            .register_stellar_asset_contract_v2(token_admin.clone())
            .address();
        let token_id_2 = env
            .register_stellar_asset_contract_v2(token_admin.clone())
            .address();

        let admin = Address::generate(&env);
        let contract_id = env.register(TipJarContract, ());
        let tipjar_client = TipJarContractClient::new(&env, &contract_id);
        tipjar_client.init(&admin);
        tipjar_client.add_token(&admin, &token_id_1);

        (env, contract_id, token_id_1, token_id_2, admin)
    }

    #[test]
    fn test_tipping_functionality_multi_token() {
        let (env, contract_id, token_id_1, token_id_2, admin) = setup();
        let tipjar_client = TipJarContractClient::new(&env, &contract_id);
        let token_client_1 = token::Client::new(&env, &token_id_1);
        let token_client_2 = token::Client::new(&env, &token_id_2);
        let token_admin_client_1 = token::StellarAssetClient::new(&env, &token_id_1);
        let token_admin_client_2 = token::StellarAssetClient::new(&env, &token_id_2);
        
        let sender = Address::generate(&env);
        let creator = Address::generate(&env);

        token_admin_client_1.mint(&sender, &1_000);
        token_admin_client_2.mint(&sender, &1_000);

        // Success for whitelisted token 1
        tipjar_client.tip(&sender, &creator, &token_id_1, &250);
        assert_eq!(token_client_1.balance(&sender), 750);
        assert_eq!(token_client_1.balance(&contract_id), 250);
        assert_eq!(tipjar_client.get_total_tips(&creator, &token_id_1), 250);

        // Failure for non-whitelisted token 2
        let result = tipjar_client.try_tip(&sender, &creator, &token_id_2, &100);
        assert!(result.is_err());

        // Success after whitelisting token 2
        tipjar_client.add_token(&admin, &token_id_2);
        tipjar_client.tip(&sender, &creator, &token_id_2, &300);
        assert_eq!(token_client_2.balance(&sender), 700);
        assert_eq!(tipjar_client.get_total_tips(&creator, &token_id_2), 300);
    }

    #[test]
    fn test_balance_tracking_and_withdraw_multi_token() {
        let (env, contract_id, token_id_1, token_id_2, admin) = setup();
        let tipjar_client = TipJarContractClient::new(&env, &contract_id);
        let token_client_1 = token::Client::new(&env, &token_id_1);
        let token_client_2 = token::Client::new(&env, &token_id_2);
        let token_admin_client_1 = token::StellarAssetClient::new(&env, &token_id_1);
        let token_admin_client_2 = token::StellarAssetClient::new(&env, &token_id_2);
        
        let sender = Address::generate(&env);
        let creator = Address::generate(&env);

        tipjar_client.add_token(&admin, &token_id_2);
        token_admin_client_1.mint(&sender, &1_000);
        token_admin_client_2.mint(&sender, &1_000);

        tipjar_client.tip(&sender, &creator, &token_id_1, &100);
        tipjar_client.tip(&sender, &creator, &token_id_2, &300);

        assert_eq!(tipjar_client.get_withdrawable_balance(&creator, &token_id_1), 100);
        assert_eq!(tipjar_client.get_withdrawable_balance(&creator, &token_id_2), 300);

        // Withdraw token 1
        tipjar_client.withdraw(&creator, &token_id_1);
        assert_eq!(tipjar_client.get_withdrawable_balance(&creator, &token_id_1), 0);
        assert_eq!(token_client_1.balance(&creator), 100);
        assert_eq!(tipjar_client.get_withdrawable_balance(&creator, &token_id_2), 300);

        // Withdraw token 2
        tipjar_client.withdraw(&creator, &token_id_2);
        assert_eq!(tipjar_client.get_withdrawable_balance(&creator, &token_id_2), 0);
        assert_eq!(token_client_2.balance(&creator), 300);
    }

    #[test]
    #[should_panic]
    fn test_invalid_tip_amount() {
        let (env, contract_id, token_id_1, _, _) = setup();
        let tipjar_client = TipJarContractClient::new(&env, &contract_id);
        let token_admin_client = token::StellarAssetClient::new(&env, &token_id_1);
        let sender = Address::generate(&env);
        let creator = Address::generate(&env);

        token_admin_client.mint(&sender, &100);
        tipjar_client.tip(&sender, &creator, &token_id_1, &0);
    }

    #[test]
    fn test_pause_unpause() {
        let (env, contract_id, token_id_1, _, admin) = setup();
        let tipjar_client = TipJarContractClient::new(&env, &contract_id);

        tipjar_client.pause(&admin);

        let sender = Address::generate(&env);
        let creator = Address::generate(&env);

        let result = tipjar_client.try_tip(&sender, &creator, &token_id_1, &100);
        assert!(result.is_err());

        tipjar_client.unpause(&admin);

        let token_admin_client = token::StellarAssetClient::new(&env, &token_id_1);
        token_admin_client.mint(&sender, &100);
        tipjar_client.tip(&sender, &creator, &token_id_1, &100);
        assert_eq!(tipjar_client.get_total_tips(&creator, &token_id_1), 100);
    }

    #[test]
    #[should_panic]
    fn test_pause_admin_only() {
        let (env, contract_id, _, _, _) = setup();
        let tipjar_client = TipJarContractClient::new(&env, &contract_id);
        let non_admin = Address::generate(&env);

        tipjar_client.pause(&non_admin);
    }
}
