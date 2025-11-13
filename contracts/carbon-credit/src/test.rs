#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_issue_and_get_credit() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CarbonCreditContract);
    let client = CarbonCreditContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let verification_contract = Address::generate(&env);
    
    // Initialize contract
    client.initialize(&admin, &verification_contract);
    
    // Issue credit
    env.mock_all_auths();
    let credit_id = client.issue_credit(
        &issuer,
        &String::from_str(&env, "PROJ-001"),
        &String::from_str(&env, "Solar Farm Project"),
        &2024,
        &1000,
    );
    
    assert_eq!(credit_id, 1);
    
    // Get credit
    let metadata = client.get_credit(&credit_id);
    assert_eq!(metadata.amount_tons, 1000);
    assert_eq!(metadata.vintage_year, 2024);
    assert_eq!(metadata.verification_status, VerificationStatus::Pending);
    
    // Check owner
    let owner = client.get_owner(&credit_id);
    assert_eq!(owner, Some(issuer));
}

#[test]
fn test_transfer_credit() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CarbonCreditContract);
    let client = CarbonCreditContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let buyer = Address::generate(&env);
    let verification_contract = Address::generate(&env);
    
    client.initialize(&admin, &verification_contract);
    
    env.mock_all_auths();
    let credit_id = client.issue_credit(
        &issuer,
        &String::from_str(&env, "PROJ-001"),
        &String::from_str(&env, "Wind Farm"),
        &2024,
        &500,
    );
    
    // Transfer credit
    client.transfer(&issuer, &buyer, &credit_id);
    
    // Check new owner
    let owner = client.get_owner(&credit_id);
    assert_eq!(owner, Some(buyer));
}

#[test]
fn test_verification_update() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CarbonCreditContract);
    let client = CarbonCreditContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let verification_contract = Address::generate(&env);
    
    client.initialize(&admin, &verification_contract);
    
    env.mock_all_auths();
    let credit_id = client.issue_credit(
        &issuer,
        &String::from_str(&env, "PROJ-001"),
        &String::from_str(&env, "Reforestation"),
        &2024,
        &2000,
    );
    
    // Update verification status
    client.update_verification(&credit_id, &VerificationStatus::Verified);
    
    // Check verification
    assert!(client.is_verified(&credit_id));
}
