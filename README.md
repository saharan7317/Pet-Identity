# 🐾 PetRegistry — On-Chain Pet Records on Stellar

> A Soroban smart contract that lets pet owners register their animals and maintain a tamper-proof medical history directly on the Stellar blockchain.

---

## 📖 Project Description

**PetRegistry** is a decentralized pet identification and health-record system built with [Soroban](https://soroban.stellar.org/), Stellar's smart contract platform. Instead of relying on paper certificates, siloed vet software, or centralised registries that can be lost or forged, PetRegistry anchors every pet's identity and medical timeline on-chain — permanent, transparent, and owner-controlled.

Whether you're a pet owner moving between cities, a vet clinic needing instant access to a patient's vaccination history, or a shelter verifying a rescue animal's background, PetRegistry gives everyone a single source of truth that nobody can tamper with.

---

## 🔍 What It Does

| Action | Who Can Call | Description |
|---|---|---|
| `register_pet` | Pet owner | Creates a new on-chain pet record and returns a unique `pet_id` |
| `add_medical_entry` | Pet owner | Appends a timestamped medical event (vaccination, checkup, surgery, etc.) |
| `transfer_ownership` | Current owner | Moves the pet record to a new owner's address |
| `get_pet` | Anyone | Reads the full pet profile by `pet_id` |
| `get_owner_pets` | Anyone | Lists all `pet_id`s belonging to a wallet address |
| `get_medical_history` | Anyone | Returns the complete ordered medical log for a pet |
| `total_pets` | Anyone | Returns the total number of pets ever registered |

All write operations require the owner's **Stellar account signature** (`require_auth`), so no one can register, update, or transfer a pet on your behalf.

---

## ✨ Features

### 🆔 Unique On-Chain Identity
Each pet is assigned an auto-incrementing `pet_id` and stored with:
- Name, species, breed, and birth year
- Optional microchip ID for cross-referencing physical tags
- Owning Stellar address
- Ledger timestamp of registration

### 🏥 Immutable Medical History
Owners can append unlimited medical entries, each recording:
- Entry type (`Vaccination`, `Checkup`, `Surgery`, `Deworming`, …)
- Description of the procedure or finding
- Attending vet's name
- Ledger-verified timestamp

Because entries are **appended only** and live on-chain, the history cannot be edited or deleted — giving vets and adopters a trustworthy audit trail.

### 🔐 Owner-Gated Writes
Every state-changing function calls `require_auth()` on the owner's address. Only the wallet that owns a pet can:
- Add medical entries
- Transfer ownership to another address

### 🔄 Ownership Transfer
Pets can be re-homed to any Stellar address in a single transaction. The contract automatically updates both the old and new owner's pet-ID index so queries stay accurate after a transfer.

### 📡 On-Chain Events
Key actions emit Soroban events (`register/pet`, `transfer/pet`, `medical/add`) that can be indexed by explorers or off-chain services for notifications and analytics.

### 🧪 Fully Tested
Three test scenarios ship with the contract:
- `test_register_and_query` — end-to-end registration and read-back
- `test_medical_history` — adding and retrieving multiple medical entries
- `test_transfer_ownership` — verifying owner index updates after transfer

---

## 🚀 Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the WASM target
rustup target add wasm32-unknown-unknown

# Install the Stellar CLI
cargo install --locked stellar-cli --features opt
```

### Build

```bash
stellar contract build
# output: target/wasm32-unknown-unknown/release/pet_registry.wasm
```

### Run Tests

```bash
cargo test
```

### Deploy to Testnet

```bash
# Fund a test account
stellar keys generate --global alice --network testnet --fund

# Deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/pet_registry.wasm \
  --source alice \
  --network testnet
```

### Invoke — Register a Pet

```bash
stellar contract invoke \
  --source alice \
  --network testnet \
  -- register_pet \
  --name "Buddy" \
  --species "Dog" \
  --breed "Labrador" \
  --birth_year 2019 \
  --microchip_id "CHIP-001"
```

### Invoke — Add a Medical Entry

```bash
stellar contract invoke \
  --source alice \
  --network testnet \
  -- add_medical_entry \
  --pet_id 1 \
  --entry_type "Vaccination" \
  --description "Annual rabies booster administered" \
  --vet_name "Dr. Patel"
```

### Invoke — Query Medical History

```bash
stellar contract invoke \
  --network testnet \
  -- get_medical_history \
  --pet_id 1
```

---

## 🗂️ Project Structure

```
pet-registry/
├── Cargo.toml          # Rust package manifest & Soroban SDK dependency
└── src/
    └── lib.rs          # Contract logic, data types, and tests
```

---

## 🛣️ Roadmap

- [ ] Vet whitelist — allow approved vets to add medical entries directly
- [ ] Breed/species validation via oracle
- [ ] Lost-pet flag that emits a public on-chain alert
- [ ] Frontend dApp (React + Freighter wallet integration)
- [ ] IPFS attachment links for storing X-ray or lab report files

---

wallet address: GBDJUYCL5GAL7PTMZBEXNDL4MWX4YSPNZUUX7RNQLCCQBHDAINT5LCFK

contract address: CCESPBGR3EZCXEBO2S7KPOSOVVQXBISWPBF4FH4LE5DAZMZYD6WKLS33

https://stellar.expert/explorer/testnet/contract/CCESPBGR3EZCXEBO2S7KPOSOVVQXBISWPBF4FH4LE5DAZMZYD6WKLS33

<img width="1919" height="863" alt="image" src="https://github.com/user-attachments/assets/c2728d99-4956-49a4-a1c9-098f2126327c" />
