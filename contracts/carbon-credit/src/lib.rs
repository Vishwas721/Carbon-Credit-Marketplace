#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, String, Symbol};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    TokenInfo,
    CreditInfo(u64), // credit_id -> CreditMetadata
    CreditOwner(u64), // credit_id -> Owner address
    OwnerCredits(Address), // owner -> Vec<credit_id>
    NextCreditId,
    VerificationContract,
}

#[derive(Clone)]
#[contracttype]
pub struct CreditMetadata {
    pub credit_id: u64,
    pub project_id: String,
    pub project_name: String,
    pub vintage_year: u32,
    pub amount_tons: u64,
    pub verification_status: VerificationStatus,
    pub issuer: Address,
    pub created_at: u64,
}

#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum VerificationStatus {
    Pending,
    Verified,
    Rejected,
}

#[contract]
pub struct CarbonCreditContract;

#[contractimpl]
impl CarbonCreditContract {
    /// Initialize the contract with admin
    pub fn initialize(env: Env, admin: Address, verification_contract: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextCreditId, &1u64);
        env.storage().instance().set(&DataKey::VerificationContract, &verification_contract);
    }

    /// Issue a new carbon credit
    pub fn issue_credit(
        env: Env,
        issuer: Address,
        project_id: String,
        project_name: String,
        vintage_year: u32,
        amount_tons: u64,
    ) -> u64 {
        issuer.require_auth();
        
        let credit_id: u64 = env.storage().instance().get(&DataKey::NextCreditId).unwrap();
        
        let metadata = CreditMetadata {
            credit_id,
            project_id,
            project_name,
            vintage_year,
            amount_tons,
            verification_status: VerificationStatus::Pending,
            issuer: issuer.clone(),
            created_at: env.ledger().timestamp(),
        };
        
        env.storage().persistent().set(&DataKey::CreditInfo(credit_id), &metadata);
        env.storage().persistent().set(&DataKey::CreditOwner(credit_id), &issuer);
        
        // Update next credit ID
        env.storage().instance().set(&DataKey::NextCreditId, &(credit_id + 1));
        
        // Emit event
        env.events().publish(
            (Symbol::new(&env, "credit_issued"),),
            (credit_id, issuer, amount_tons)
        );
        
        credit_id
    }

    /// Update verification status (called by verification contract)
    pub fn update_verification(
        env: Env,
        credit_id: u64,
        status: VerificationStatus,
    ) {
        let verification_contract: Address = env.storage()
            .instance()
            .get(&DataKey::VerificationContract)
            .unwrap();
        
        verification_contract.require_auth();
        
        let mut metadata: CreditMetadata = env.storage()
            .persistent()
            .get(&DataKey::CreditInfo(credit_id))
            .unwrap();
        
        metadata.verification_status = status.clone();
        env.storage().persistent().set(&DataKey::CreditInfo(credit_id), &metadata);
        
        env.events().publish(
            (Symbol::new(&env, "verification_updated"),),
            (credit_id, status)
        );
    }

    /// Transfer credit ownership
    pub fn transfer(env: Env, from: Address, to: Address, credit_id: u64) {
        from.require_auth();
        
        let current_owner: Address = env.storage()
            .persistent()
            .get(&DataKey::CreditOwner(credit_id))
            .unwrap();
        
        if current_owner != from {
            panic!("Not the owner");
        }
        
        env.storage().persistent().set(&DataKey::CreditOwner(credit_id), &to);
        
        env.events().publish(
            (Symbol::new(&env, "credit_transferred"),),
            (credit_id, from, to)
        );
    }

    /// Retire a carbon credit (permanently removes from circulation)
    pub fn retire_credit(env: Env, owner: Address, credit_id: u64) {
        owner.require_auth();
        
        let current_owner: Address = env.storage()
            .persistent()
            .get(&DataKey::CreditOwner(credit_id))
            .unwrap();
        
        if current_owner != owner {
            panic!("Not the owner");
        }
        
        let metadata: CreditMetadata = env.storage()
            .persistent()
            .get(&DataKey::CreditInfo(credit_id))
            .unwrap();
        
        if metadata.verification_status != VerificationStatus::Verified {
            panic!("Only verified credits can be retired");
        }
        
        // Mark as retired by removing ownership
        env.storage().persistent().remove(&DataKey::CreditOwner(credit_id));
        
        env.events().publish(
            (Symbol::new(&env, "credit_retired"),),
            (credit_id, owner, metadata.amount_tons)
        );
    }

    /// Get credit metadata
    pub fn get_credit(env: Env, credit_id: u64) -> CreditMetadata {
        env.storage()
            .persistent()
            .get(&DataKey::CreditInfo(credit_id))
            .unwrap()
    }

    /// Get credit owner
    pub fn get_owner(env: Env, credit_id: u64) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::CreditOwner(credit_id))
    }

    /// Check if credit is verified
    pub fn is_verified(env: Env, credit_id: u64) -> bool {
        let metadata: CreditMetadata = env.storage()
            .persistent()
            .get(&DataKey::CreditInfo(credit_id))
            .unwrap();
        
        metadata.verification_status == VerificationStatus::Verified
    }
}

mod test;
