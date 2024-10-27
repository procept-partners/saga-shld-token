use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault};
use near_sdk::json_types::U128;
use ethabi::ethereum_types::H160;
use secp256k1::verify;

#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKey {
    Tokens,
    TokenOwners,
    AccountTokens,
    Proposals,
    ProposalVoters { proposal_id: u64 },
}

// Main SHLDContract struct with necessary fields
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct SHLDContract {
    tokens: LookupMap<AccountId, Token>,
    token_owners: UnorderedSet<AccountId>,
    account_tokens: LookupMap<AccountId, String>,
    proposals: UnorderedMap<u64, Proposal>,
    next_proposal_id: u64,
    members_registry: UnorderedSet<String>,
    next_nft_number: u64,
    current_minting_round: u64,
    minting_order_in_round: u64,
    contract_owner: AccountId,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Token {
    owner_id: AccountId,
    metadata: TokenMetadata,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    ticker_title: String,
    avatar_name: Option<String>,
    profile_description: Option<String>,
    governance_role: String,
    profile_image_url: Option<String>,
    near_account_id: AccountId,
    ethereum_address: Option<H160>,
    cooperative_id: String,
    did: Option<String>,
    verification_status: String,
    minting_timestamp: u64,
    nft_number: u64,
    minting_round: u64,
    minting_order_in_round: u64,
    unique_hash: String,
    member_titles: Vec<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OwnershipProof {
    near_account_id: AccountId,
    token_hash: String,
    signature: Vec<u8>,
}

#[near_bindgen]
impl SHLDContract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            tokens: LookupMap::new(StorageKey::Tokens),
            token_owners: UnorderedSet::new(StorageKey::TokenOwners),
            account_tokens: LookupMap::new(StorageKey::AccountTokens),
            proposals: UnorderedMap::new(StorageKey::Proposals),
            next_proposal_id: 0,
            members_registry: UnorderedSet::new(b"m"),
            next_nft_number: 0,
            current_minting_round: 1,
            minting_order_in_round: 0,
            contract_owner: owner_id,
        }
    }

    pub fn increment_minting_round(&mut self) {
        require!(
            env::predecessor_account_id() == self.contract_owner,
            "Only the contract owner can increment the minting round"
        );
        self.current_minting_round += 1;
        self.minting_order_in_round = 0;
    }

    pub fn mint(&mut self, account_id: AccountId, metadata: TokenMetadata) {
        require!(!self.tokens.contains_key(&account_id), "Token already exists for this account");

        self.next_nft_number += 1;
        self.minting_order_in_round += 1;

        let unique_hash = self.generate_unique_hash(&metadata.cooperative_id, self.next_nft_number);

        let token = Token {
            owner_id: account_id.clone(),
            metadata: TokenMetadata {
                nft_number: self.next_nft_number,
                minting_round: self.current_minting_round,
                minting_order_in_round: self.minting_order_in_round,
                unique_hash: unique_hash.clone(),
                ..metadata
            },
        };

        self.tokens.insert(&account_id, &token);
        self.token_owners.insert(&account_id);
        self.account_tokens.insert(&account_id, &unique_hash); // Link NEAR account to SHLD token hash
    }

    pub fn link_shld_token(&mut self, account_id: AccountId, token_hash: String) {
        self.account_tokens.insert(&account_id, &token_hash);
    }

    pub fn update_avatar_name(&mut self, account_id: AccountId, new_avatar_name: String) {
        let mut token = self.tokens.get(&account_id).expect("Token does not exist for this account");
        token.metadata.avatar_name = Some(new_avatar_name);
        self.tokens.insert(&account_id, &token);
    }

    pub fn revoke_nft(&mut self, account_id: AccountId) {
        require!(env::predecessor_account_id() == self.contract_owner, "Only the contract owner can revoke NFTs");

        let token = self.tokens.remove(&account_id).expect("Token does not exist for this account");
        self.token_owners.remove(&account_id);
        self.members_registry.remove(&token.metadata.cooperative_id);
        self.account_tokens.remove(&account_id);
    }

    pub fn generate_ownership_proof(&self, account_id: AccountId) -> OwnershipProof {
        let token_hash = self.account_tokens.get(&account_id).expect("No SHLD token linked to this account");

        let message = format!("{} owns SHLD token {}", account_id, token_hash);
        let message_hash = env::sha256(message.as_bytes());
        let signature = env::sign(&message_hash);

        OwnershipProof {
            near_account_id: account_id,
            token_hash,
            signature,
        }
    }

    fn generate_unique_hash(&self, cooperative_id: &String, nft_number: u64) -> String {
        format!("{}-{}", cooperative_id, nft_number)
    }

    pub fn token_metadata(&self, account_id: AccountId) -> Option<TokenMetadata> {
        self.tokens.get(&account_id).map(|token| token.metadata.clone())
    }

    pub fn is_token_owner(&self, account_id: AccountId) -> bool {
        self.token_owners.contains(&account_id)
    }

    pub fn governance_role(&self, account_id: AccountId) -> Option<String> {
        self.tokens.get(&account_id).map(|token| token.metadata.governance_role.clone())
    }
}

// SPDX-License-Identifier: MIT
// Near contract for verifying SHLD ownership and emitting event for Rainbow Bridge
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId};

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct SHLDOwnershipVerifier {
    authorized_signer: AccountId,
}

#[near_bindgen]
impl SHLDOwnershipVerifier {
    #[init]
    pub fn new(authorized_signer: AccountId) -> Self {
        Self { authorized_signer }
    }

    pub fn verify_ownership(
        &self,
        account_id: AccountId,
        token_hash: String,
        signature: Vec<u8>
    ) -> bool {
        // Verification logic here (omitted for brevity)
        
        env::log_str(&format!(
            "SHLDOwnershipVerified: {{ account_id: {}, token_hash: {} }}",
            account_id, token_hash
        ));
        true
    }
}
