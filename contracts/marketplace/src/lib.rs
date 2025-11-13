#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol, Vec};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    CarbonCreditContract,
    PaymentToken,
    Listing(u64), // listing_id -> Listing
    NextListingId,
    ActiveListings,
    MarketplaceFee, // Percentage fee (e.g., 250 = 2.5%)
}

#[derive(Clone)]
#[contracttype]
pub struct Listing {
    pub listing_id: u64,
    pub credit_id: u64,
    pub seller: Address,
    pub price_per_ton: i128,
    pub amount_tons: u64,
    pub status: ListingStatus,
    pub created_at: u64,
}

#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum ListingStatus {
    Active,
    Sold,
    Cancelled,
}

#[contract]
pub struct MarketplaceContract;

#[contractimpl]
impl MarketplaceContract {
    /// Initialize the marketplace
    pub fn initialize(
        env: Env,
        admin: Address,
        carbon_credit_contract: Address,
        payment_token: Address,
        marketplace_fee: u32, // Basis points (e.g., 250 = 2.5%)
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CarbonCreditContract, &carbon_credit_contract);
        env.storage().instance().set(&DataKey::PaymentToken, &payment_token);
        env.storage().instance().set(&DataKey::MarketplaceFee, &marketplace_fee);
        env.storage().instance().set(&DataKey::NextListingId, &1u64);
        
        let active_listings: Vec<u64> = Vec::new(&env);
        env.storage().instance().set(&DataKey::ActiveListings, &active_listings);
    }

    /// Create a listing for a carbon credit
    pub fn create_listing(
        env: Env,
        seller: Address,
        credit_id: u64,
        price_per_ton: i128,
        amount_tons: u64,
    ) -> u64 {
        seller.require_auth();
        
        if price_per_ton <= 0 {
            panic!("Price must be positive");
        }
        
        if amount_tons == 0 {
            panic!("Amount must be positive");
        }
        
        let listing_id: u64 = env.storage().instance().get(&DataKey::NextListingId).unwrap();
        
        let listing = Listing {
            listing_id,
            credit_id,
            seller: seller.clone(),
            price_per_ton,
            amount_tons,
            status: ListingStatus::Active,
            created_at: env.ledger().timestamp(),
        };
        
        env.storage().persistent().set(&DataKey::Listing(listing_id), &listing);
        
        // Add to active listings
        let mut active_listings: Vec<u64> = env.storage().instance().get(&DataKey::ActiveListings).unwrap();
        active_listings.push_back(listing_id);
        env.storage().instance().set(&DataKey::ActiveListings, &active_listings);
        
        // Update next listing ID
        env.storage().instance().set(&DataKey::NextListingId, &(listing_id + 1));
        
        env.events().publish(
            (Symbol::new(&env, "listing_created"),),
            (listing_id, credit_id, seller, price_per_ton, amount_tons)
        );
        
        listing_id
    }

    /// Purchase a carbon credit from the marketplace
    pub fn buy_credit(env: Env, buyer: Address, listing_id: u64) {
        buyer.require_auth();
        
        let mut listing: Listing = env.storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .unwrap();
        
        if listing.status != ListingStatus::Active {
            panic!("Listing is not active");
        }
        
        let total_price = listing.price_per_ton * listing.amount_tons as i128;
        
        // Calculate marketplace fee
        let marketplace_fee: u32 = env.storage().instance().get(&DataKey::MarketplaceFee).unwrap();
        let fee_amount = (total_price * marketplace_fee as i128) / 10000;
        let seller_amount = total_price - fee_amount;
        
        let payment_token: Address = env.storage().instance().get(&DataKey::PaymentToken).unwrap();
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        
        // Transfer payment token from buyer to seller
        let token_client = token::Client::new(&env, &payment_token);
        token_client.transfer(&buyer, &listing.seller, &seller_amount);
        
        // Transfer fee to marketplace admin
        if fee_amount > 0 {
            token_client.transfer(&buyer, &admin, &fee_amount);
        }
        
        // Transfer carbon credit ownership
        let carbon_credit_contract: Address = env.storage()
            .instance()
            .get(&DataKey::CarbonCreditContract)
            .unwrap();
        
        // Mark listing as sold
        listing.status = ListingStatus::Sold;
        env.storage().persistent().set(&DataKey::Listing(listing_id), &listing);
        
        // Remove from active listings
        Self::remove_from_active_listings(&env, listing_id);
        
        env.events().publish(
            (Symbol::new(&env, "credit_purchased"),),
            (listing_id, buyer, listing.seller.clone(), total_price)
        );
    }

    /// Cancel a listing
    pub fn cancel_listing(env: Env, seller: Address, listing_id: u64) {
        seller.require_auth();
        
        let mut listing: Listing = env.storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .unwrap();
        
        if listing.seller != seller {
            panic!("Not the seller");
        }
        
        if listing.status != ListingStatus::Active {
            panic!("Listing is not active");
        }
        
        listing.status = ListingStatus::Cancelled;
        env.storage().persistent().set(&DataKey::Listing(listing_id), &listing);
        
        // Remove from active listings
        Self::remove_from_active_listings(&env, listing_id);
        
        env.events().publish(
            (Symbol::new(&env, "listing_cancelled"),),
            (listing_id, seller)
        );
    }

    /// Update listing price
    pub fn update_price(env: Env, seller: Address, listing_id: u64, new_price: i128) {
        seller.require_auth();
        
        if new_price <= 0 {
            panic!("Price must be positive");
        }
        
        let mut listing: Listing = env.storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .unwrap();
        
        if listing.seller != seller {
            panic!("Not the seller");
        }
        
        if listing.status != ListingStatus::Active {
            panic!("Listing is not active");
        }
        
        listing.price_per_ton = new_price;
        env.storage().persistent().set(&DataKey::Listing(listing_id), &listing);
        
        env.events().publish(
            (Symbol::new(&env, "price_updated"),),
            (listing_id, new_price)
        );
    }

    /// Get listing details
    pub fn get_listing(env: Env, listing_id: u64) -> Listing {
        env.storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .unwrap()
    }

    /// Get all active listings
    pub fn get_active_listings(env: Env) -> Vec<u64> {
        env.storage().instance().get(&DataKey::ActiveListings).unwrap()
    }

    /// Get marketplace fee
    pub fn get_marketplace_fee(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::MarketplaceFee).unwrap()
    }

    /// Update marketplace fee (admin only)
    pub fn update_marketplace_fee(env: Env, admin: Address, new_fee: u32) {
        admin.require_auth();
        
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Not authorized");
        }
        
        if new_fee > 1000 {
            panic!("Fee too high (max 10%)");
        }
        
        env.storage().instance().set(&DataKey::MarketplaceFee, &new_fee);
        
        env.events().publish(
            (Symbol::new(&env, "fee_updated"),),
            new_fee
        );
    }

    // Helper function to remove listing from active listings
    fn remove_from_active_listings(env: &Env, listing_id: u64) {
        let active_listings: Vec<u64> = env.storage().instance().get(&DataKey::ActiveListings).unwrap();
        let mut new_active_listings: Vec<u64> = Vec::new(&env);
        
        for id in active_listings.iter() {
            if id != listing_id {
                new_active_listings.push_back(id);
            }
        }
        
        env.storage().instance().set(&DataKey::ActiveListings, &new_active_listings);
    }
}

mod test;
