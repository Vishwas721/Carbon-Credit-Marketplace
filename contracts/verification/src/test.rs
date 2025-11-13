#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_initialize_and_add_verifier() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VerificationContract);
    let client = VerificationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let carbon_credit_contract = Address::generate(&env);
    let verifier = Address::generate(&env);
    
    // Initialize
    client.initialize(&admin, &carbon_credit_contract);
    
    // Add verifier
    env.mock_all_auths();
    client.add_verifier(&admin, &verifier);
    
    // Check if verifier was added
    assert!(client.is_verifier(&verifier));
    
    let verifiers = client.get_verifiers();
    assert_eq!(verifiers.len(), 1);
}

#[test]
fn test_submit_and_approve_verification() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VerificationContract);
    let client = VerificationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let carbon_credit_contract = Address::generate(&env);
    let verifier = Address::generate(&env);
    let requester = Address::generate(&env);
    
    client.initialize(&admin, &carbon_credit_contract);
    
    env.mock_all_auths();
    client.add_verifier(&admin, &verifier);
    
    // Submit verification request
    let credit_id = 1u64;
    client.submit_verification(
        &requester,
        &credit_id,
        &String::from_str(&env, "PROJ-001"),
        &String::from_str(&env, "https://evidence.example.com/proj001"),
    );
    
    // Assign verifier
    client.assign_verifier(&admin, &credit_id, &verifier);
    
    // Approve verification
    client.approve_verification(
        &verifier,
        &credit_id,
        &String::from_str(&env, "All documentation verified"),
    );
    
    // Check request status
    let request = client.get_request(&credit_id);
    assert_eq!(request.status, RequestStatus::Approved);
}

#[test]
fn test_reject_verification() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VerificationContract);
    let client = VerificationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let carbon_credit_contract = Address::generate(&env);
    let verifier = Address::generate(&env);
    let requester = Address::generate(&env);
    
    client.initialize(&admin, &carbon_credit_contract);
    
    env.mock_all_auths();
    client.add_verifier(&admin, &verifier);
    
    let credit_id = 2u64;
    client.submit_verification(
        &requester,
        &credit_id,
        &String::from_str(&env, "PROJ-002"),
        &String::from_str(&env, "https://evidence.example.com/proj002"),
    );
    
    client.assign_verifier(&admin, &credit_id, &verifier);
    
    // Reject verification
    client.reject_verification(
        &verifier,
        &credit_id,
        &String::from_str(&env, "Insufficient documentation"),
    );
    
    let request = client.get_request(&credit_id);
    assert_eq!(request.status, RequestStatus::Rejected);
}
