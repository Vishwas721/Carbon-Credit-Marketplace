#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Verifiers,
    VerificationRequest(u64), // credit_id -> VerificationRequest
    CarbonCreditContract,
}

#[derive(Clone)]
#[contracttype]
pub struct VerificationRequest {
    pub credit_id: u64,
    pub requester: Address,
    pub project_id: String,
    pub evidence_uri: String,
    pub status: RequestStatus,
    pub verifier: Option<Address>,
    pub verified_at: Option<u64>,
    pub notes: String,
}

#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum RequestStatus {
    Pending,
    UnderReview,
    Approved,
    Rejected,
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum VerificationStatus {
    Pending,
    Verified,
    Rejected,
}

#[contract]
pub struct VerificationContract;

#[contractimpl]
impl VerificationContract {
    /// Initialize the contract
    pub fn initialize(env: Env, admin: Address, carbon_credit_contract: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CarbonCreditContract, &carbon_credit_contract);
        
        // Initialize empty verifiers list
        let verifiers: Vec<Address> = Vec::new(&env);
        env.storage().instance().set(&DataKey::Verifiers, &verifiers);
    }

    /// Add a verifier (admin only)
    pub fn add_verifier(env: Env, admin: Address, verifier: Address) {
        admin.require_auth();
        
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Not authorized");
        }
        
        let mut verifiers: Vec<Address> = env.storage().instance().get(&DataKey::Verifiers).unwrap();
        verifiers.push_back(verifier.clone());
        env.storage().instance().set(&DataKey::Verifiers, &verifiers);
        
        env.events().publish(
            (Symbol::new(&env, "verifier_added"),),
            verifier
        );
    }

    /// Remove a verifier (admin only)
    pub fn remove_verifier(env: Env, admin: Address, verifier: Address) {
        admin.require_auth();
        
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Not authorized");
        }
        
        let mut verifiers: Vec<Address> = env.storage().instance().get(&DataKey::Verifiers).unwrap();
        let mut new_verifiers: Vec<Address> = Vec::new(&env);
        
        for v in verifiers.iter() {
            if v != verifier {
                new_verifiers.push_back(v);
            }
        }
        
        env.storage().instance().set(&DataKey::Verifiers, &new_verifiers);
        
        env.events().publish(
            (Symbol::new(&env, "verifier_removed"),),
            verifier
        );
    }

    /// Submit a verification request
    pub fn submit_verification(
        env: Env,
        requester: Address,
        credit_id: u64,
        project_id: String,
        evidence_uri: String,
    ) {
        requester.require_auth();
        
        let request = VerificationRequest {
            credit_id,
            requester,
            project_id,
            evidence_uri,
            status: RequestStatus::Pending,
            verifier: None,
            verified_at: None,
            notes: String::from_str(&env, ""),
        };
        
        env.storage().persistent().set(&DataKey::VerificationRequest(credit_id), &request);
        
        env.events().publish(
            (Symbol::new(&env, "verification_submitted"),),
            credit_id
        );
    }

    /// Assign a verifier to review a request (admin or verifier)
    pub fn assign_verifier(env: Env, caller: Address, credit_id: u64, verifier: Address) {
        caller.require_auth();
        
        // Check if caller is admin or a verifier
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let verifiers: Vec<Address> = env.storage().instance().get(&DataKey::Verifiers).unwrap();
        
        let mut is_authorized = caller == admin;
        if !is_authorized {
            for v in verifiers.iter() {
                if v == caller {
                    is_authorized = true;
                    break;
                }
            }
        }
        
        if !is_authorized {
            panic!("Not authorized");
        }
        
        let mut request: VerificationRequest = env.storage()
            .persistent()
            .get(&DataKey::VerificationRequest(credit_id))
            .unwrap();
        
        request.status = RequestStatus::UnderReview;
        request.verifier = Some(verifier.clone());
        
        env.storage().persistent().set(&DataKey::VerificationRequest(credit_id), &request);
        
        env.events().publish(
            (Symbol::new(&env, "verifier_assigned"),),
            (credit_id, verifier)
        );
    }

    /// Approve a verification request (verifier only)
    pub fn approve_verification(env: Env, verifier: Address, credit_id: u64, notes: String) {
        verifier.require_auth();
        
        let mut request: VerificationRequest = env.storage()
            .persistent()
            .get(&DataKey::VerificationRequest(credit_id))
            .unwrap();
        
        // Check if verifier is assigned to this request
        if request.verifier != Some(verifier.clone()) {
            panic!("Not the assigned verifier");
        }
        
        request.status = RequestStatus::Approved;
        request.verified_at = Some(env.ledger().timestamp());
        request.notes = notes;
        
        env.storage().persistent().set(&DataKey::VerificationRequest(credit_id), &request);
        
        // Update carbon credit contract
        let carbon_credit_contract: Address = env.storage()
            .instance()
            .get(&DataKey::CarbonCreditContract)
            .unwrap();
        
        // Call update_verification on carbon credit contract
        // This requires proper cross-contract call setup
        env.events().publish(
            (Symbol::new(&env, "verification_approved"),),
            (credit_id, verifier)
        );
    }

    /// Reject a verification request (verifier only)
    pub fn reject_verification(env: Env, verifier: Address, credit_id: u64, reason: String) {
        verifier.require_auth();
        
        let mut request: VerificationRequest = env.storage()
            .persistent()
            .get(&DataKey::VerificationRequest(credit_id))
            .unwrap();
        
        // Check if verifier is assigned to this request
        if request.verifier != Some(verifier.clone()) {
            panic!("Not the assigned verifier");
        }
        
        request.status = RequestStatus::Rejected;
        request.verified_at = Some(env.ledger().timestamp());
        request.notes = reason;
        
        env.storage().persistent().set(&DataKey::VerificationRequest(credit_id), &request);
        
        env.events().publish(
            (Symbol::new(&env, "verification_rejected"),),
            (credit_id, verifier)
        );
    }

    /// Get verification request details
    pub fn get_request(env: Env, credit_id: u64) -> VerificationRequest {
        env.storage()
            .persistent()
            .get(&DataKey::VerificationRequest(credit_id))
            .unwrap()
    }

    /// Check if address is a verifier
    pub fn is_verifier(env: Env, address: Address) -> bool {
        let verifiers: Vec<Address> = env.storage().instance().get(&DataKey::Verifiers).unwrap();
        
        for verifier in verifiers.iter() {
            if verifier == address {
                return true;
            }
        }
        
        false
    }

    /// Get all verifiers
    pub fn get_verifiers(env: Env) -> Vec<Address> {
        env.storage().instance().get(&DataKey::Verifiers).unwrap()
    }
}

mod test;
