#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, String, Symbol, Vec, symbol_short,
};

// ─── Data Types ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Pet(u64),           // pet_id → PetRecord
    OwnerPets(Address), // owner → Vec<pet_id>
    NextId,             // auto-increment counter
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PetRecord {
    pub pet_id:       u64,
    pub owner:        Address,
    pub name:         String,
    pub species:      String,   // e.g. "Dog", "Cat", "Bird"
    pub breed:        String,
    pub birth_year:   u32,
    pub microchip_id: String,   // optional; pass empty string if none
    pub registered_at: u64,     // ledger timestamp
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MedicalEntry {
    pub pet_id:      u64,
    pub entry_type:  String,    // "Vaccination", "Checkup", "Surgery", etc.
    pub description: String,
    pub vet_name:    String,
    pub recorded_at: u64,       // ledger timestamp
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MedKey {
    History(u64), // pet_id → Vec<MedicalEntry>
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct PetRegistryContract;

#[contractimpl]
impl PetRegistryContract {

    // ── Registration ─────────────────────────────────────────────────────────

    /// Register a new pet. Returns the assigned pet_id.
    /// The caller (owner) must sign this transaction.
    pub fn register_pet(
        env:          Env,
        owner:        Address,
        name:         String,
        species:      String,
        breed:        String,
        birth_year:   u32,
        microchip_id: String,
    ) -> u64 {
        // Require the owner to have authorized this call
        owner.require_auth();

        // Fetch & bump the global ID counter
        let pet_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextId)
            .unwrap_or(0_u64);

        let new_id = pet_id + 1;
        env.storage().instance().set(&DataKey::NextId, &new_id);

        let now = env.ledger().timestamp();

        // Build the pet record
        let record = PetRecord {
            pet_id: new_id,
            owner: owner.clone(),
            name,
            species,
            breed,
            birth_year,
            microchip_id,
            registered_at: now,
        };

        // Persist the pet record
        env.storage()
            .persistent()
            .set(&DataKey::Pet(new_id), &record);

        // Update owner → [pet_ids] index
        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::OwnerPets(owner.clone()))
            .unwrap_or(Vec::new(&env));
        ids.push_back(new_id);
        env.storage()
            .persistent()
            .set(&DataKey::OwnerPets(owner), &ids);

        // Emit event
        env.events().publish(
            (symbol_short!("register"), symbol_short!("pet")),
            new_id,
        );

        new_id
    }

    // ── Ownership Transfer ────────────────────────────────────────────────────

    /// Transfer pet ownership to a new address.
    /// Only the current owner can call this.
    pub fn transfer_ownership(
        env:       Env,
        pet_id:    u64,
        new_owner: Address,
    ) {
        let mut record: PetRecord = Self::get_pet_record(&env, pet_id);

        // Only current owner may transfer
        record.owner.require_auth();

        let old_owner = record.owner.clone();

        // Remove pet_id from old owner's list
        let mut old_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::OwnerPets(old_owner.clone()))
            .unwrap_or(Vec::new(&env));

        // rebuild without this pet_id
        let mut updated_old: Vec<u64> = Vec::new(&env);
        for i in 0..old_ids.len() {
            let id = old_ids.get(i).unwrap();
            if id != pet_id {
                updated_old.push_back(id);
            }
        }
        env.storage()
            .persistent()
            .set(&DataKey::OwnerPets(old_owner), &updated_old);

        // Add to new owner's list
        let mut new_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::OwnerPets(new_owner.clone()))
            .unwrap_or(Vec::new(&env));
        new_ids.push_back(pet_id);
        env.storage()
            .persistent()
            .set(&DataKey::OwnerPets(new_owner.clone()), &new_ids);

        // Update the record
        record.owner = new_owner.clone();
        env.storage()
            .persistent()
            .set(&DataKey::Pet(pet_id), &record);

        env.events().publish(
            (symbol_short!("transfer"), symbol_short!("pet")),
            (pet_id, new_owner),
        );
    }

    // ── Medical History ───────────────────────────────────────────────────────

    /// Add a medical history entry for a pet.
    /// Only the pet owner may add entries.
    pub fn add_medical_entry(
        env:         Env,
        pet_id:      u64,
        entry_type:  String,
        description: String,
        vet_name:    String,
    ) {
        let record: PetRecord = Self::get_pet_record(&env, pet_id);
        record.owner.require_auth();

        let entry = MedicalEntry {
            pet_id,
            entry_type,
            description,
            vet_name,
            recorded_at: env.ledger().timestamp(),
        };

        let mut history: Vec<MedicalEntry> = env
            .storage()
            .persistent()
            .get(&MedKey::History(pet_id))
            .unwrap_or(Vec::new(&env));

        history.push_back(entry);

        env.storage()
            .persistent()
            .set(&MedKey::History(pet_id), &history);

        env.events().publish(
            (symbol_short!("medical"), symbol_short!("add")),
            pet_id,
        );
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    /// Fetch a single pet record by ID.
    pub fn get_pet(env: Env, pet_id: u64) -> PetRecord {
        Self::get_pet_record(&env, pet_id)
    }

    /// Fetch all pet IDs owned by an address.
    pub fn get_owner_pets(env: Env, owner: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::OwnerPets(owner))
            .unwrap_or(Vec::new(&env))
    }

    /// Fetch the full medical history for a pet.
    pub fn get_medical_history(env: Env, pet_id: u64) -> Vec<MedicalEntry> {
        // Confirm the pet exists
        Self::get_pet_record(&env, pet_id);

        env.storage()
            .persistent()
            .get(&MedKey::History(pet_id))
            .unwrap_or(Vec::new(&env))
    }

    /// Total number of pets ever registered.
    pub fn total_pets(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextId)
            .unwrap_or(0_u64)
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn get_pet_record(env: &Env, pet_id: u64) -> PetRecord {
        env.storage()
            .persistent()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found")
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn make_str(env: &Env, s: &str) -> String {
        String::from_str(env, s)
    }

    #[test]
    fn test_register_and_query() {
        let env  = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetRegistryContract);
        let client      = PetRegistryContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &make_str(&env, "Buddy"),
            &make_str(&env, "Dog"),
            &make_str(&env, "Labrador"),
            &2019_u32,
            &make_str(&env, "CHIP-001"),
        );

        assert_eq!(pet_id, 1);
        assert_eq!(client.total_pets(), 1);

        let record = client.get_pet(&pet_id);
        assert_eq!(record.name,    make_str(&env, "Buddy"));
        assert_eq!(record.species, make_str(&env, "Dog"));
        assert_eq!(record.owner,   owner);

        let ids = client.get_owner_pets(&owner);
        assert_eq!(ids.len(), 1);
        assert_eq!(ids.get(0).unwrap(), 1_u64);
    }

    #[test]
    fn test_medical_history() {
        let env  = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetRegistryContract);
        let client      = PetRegistryContractClient::new(&env, &contract_id);

        let owner  = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &make_str(&env, "Luna"),
            &make_str(&env, "Cat"),
            &make_str(&env, "Siamese"),
            &2021_u32,
            &make_str(&env, ""),
        );

        client.add_medical_entry(
            &pet_id,
            &make_str(&env, "Vaccination"),
            &make_str(&env, "Annual rabies booster"),
            &make_str(&env, "Dr. Patel"),
        );
        client.add_medical_entry(
            &pet_id,
            &make_str(&env, "Checkup"),
            &make_str(&env, "Routine health check — all clear"),
            &make_str(&env, "Dr. Patel"),
        );

        let history = client.get_medical_history(&pet_id);
        assert_eq!(history.len(), 2);
        assert_eq!(history.get(0).unwrap().entry_type, make_str(&env, "Vaccination"));
    }

    #[test]
    fn test_transfer_ownership() {
        let env  = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetRegistryContract);
        let client      = PetRegistryContractClient::new(&env, &contract_id);

        let owner     = Address::generate(&env);
        let new_owner = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &make_str(&env, "Max"),
            &make_str(&env, "Dog"),
            &make_str(&env, "Beagle"),
            &2020_u32,
            &make_str(&env, "CHIP-MAX"),
        );

        client.transfer_ownership(&pet_id, &new_owner);

        let record = client.get_pet(&pet_id);
        assert_eq!(record.owner, new_owner);

        // old owner should have no pets
        let old_ids = client.get_owner_pets(&owner);
        assert_eq!(old_ids.len(), 0);

        // new owner should have the pet
        let new_ids = client.get_owner_pets(&new_owner);
        assert_eq!(new_ids.len(), 1);
    }
}