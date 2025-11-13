#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

fn create_token_contract<'a>(env: &Env, admin: &Address) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let token_contract_id = env.register_stellar_asset_contract_v2(admin.clone());
    let client = token::Client::new(&env, &token_contract_id.address());
    let admin_client = token::StellarAssetClient::new(&env, &token_contract_id.address());
    (client, admin_client)
}

#[test]
fn test_create_listing() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MarketplaceContract);
    let client = MarketplaceContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let carbon_credit_contract = Address::generate(&env);
    let payment_token = Address::generate(&env);
    
    // Initialize marketplace
    client.initialize(&admin, &carbon_credit_contract, &payment_token, &250);
    
    // Create listing
    env.mock_all_auths();
    let listing_id = client.create_listing(&seller, &1, &100, &500);
    
    assert_eq!(listing_id, 1);
    
    // Get listing
    let listing = client.get_listing(&listing_id);
    assert_eq!(listing.credit_id, 1);
    assert_eq!(listing.price_per_ton, 100);
    assert_eq!(listing.amount_tons, 500);
    assert_eq!(listing.status, ListingStatus::Active);
}

#[test]
fn test_buy_credit() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    
    // Create token contract
    let (token_client, token_admin) = create_token_contract(&env, &admin);
    let token_address = token_client.address.clone();
    
    // Mint tokens to buyer
    token_admin.mint(&buyer, &100000);
    
    let carbon_credit_contract = Address::generate(&env);
    
    // Create marketplace
    let contract_id = env.register_contract(None, MarketplaceContract);
    let client = MarketplaceContractClient::new(&env, &contract_id);
    
    client.initialize(&admin, &carbon_credit_contract, &token_address, &250);
    
    // Create listing
    let listing_id = client.create_listing(&seller, &1, &100, &500);
    
    // Buy credit
    client.buy_credit(&buyer, &listing_id);
    
    // Check listing status
    let listing = client.get_listing(&listing_id);
    assert_eq!(listing.status, ListingStatus::Sold);
    
    // Check seller balance (50000 - 2.5% fee = 48750)
    let seller_balance = token_client.balance(&seller);
    assert_eq!(seller_balance, 48750);
    
    // Check admin received fee
    let admin_balance = token_client.balance(&admin);
    assert_eq!(admin_balance, 1250);
}

#[test]
fn test_cancel_listing() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MarketplaceContract);
    let client = MarketplaceContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let carbon_credit_contract = Address::generate(&env);
    let payment_token = Address::generate(&env);
    
    client.initialize(&admin, &carbon_credit_contract, &payment_token, &250);
    
    env.mock_all_auths();
    let listing_id = client.create_listing(&seller, &1, &100, &500);
    
    // Cancel listing
    client.cancel_listing(&seller, &listing_id);
    
    // Check status
    let listing = client.get_listing(&listing_id);
    assert_eq!(listing.status, ListingStatus::Cancelled);
}

#[test]
fn test_update_price() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MarketplaceContract);
    let client = MarketplaceContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let carbon_credit_contract = Address::generate(&env);
    let payment_token = Address::generate(&env);
    
    client.initialize(&admin, &carbon_credit_contract, &payment_token, &250);
    
    env.mock_all_auths();
    let listing_id = client.create_listing(&seller, &1, &100, &500);
    
    // Update price
    client.update_price(&seller, &listing_id, &150);
    
    // Check new price
    let listing = client.get_listing(&listing_id);
    assert_eq!(listing.price_per_ton, 150);
}
