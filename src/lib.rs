use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct SHLDToken {
    owner_id: AccountId,
    tokens: UnorderedSet<TokenId>,
    token_to_owner: LookupMap<TokenId, AccountId>,
    token_metadata: LookupMap<TokenId, TokenMetadata>,
    governance_rights: LookupMap<TokenId, GovernanceRights>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenMetadata {
    title: Option<String>,
    description: Option<String>,
    awarded_titles: Vec<String>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct GovernanceRights {
    fyre: u128,
    global_mana: u128,
    project_mana: u128,
    allocated_project_mana: u128,
}

pub type TokenId = u64;

#[near_bindgen]
impl SHLDToken {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            tokens: UnorderedSet::new(b"t"),
            token_to_owner: LookupMap::new(b"to"),
            token_metadata: LookupMap::new(b"tm"),
            governance_rights: LookupMap::new(b"gr"),
        }
    }

    pub fn mint(&mut self, token_id: TokenId, metadata: TokenMetadata, governance_rights: GovernanceRights) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only the owner can mint tokens");
        assert!(!self.tokens.contains(&token_id), "Token already exists");

        self.tokens.insert(&token_id);
        self.token_to_owner.insert(&token_id, &env::predecessor_account_id());
        self.token_metadata.insert(&token_id, &metadata);
        self.governance_rights.insert(&token_id, &governance_rights);
    }

    pub fn burn(&mut self, token_id: TokenId) {
        let owner_id = self.token_to_owner.get(&token_id).expect("Token not found");
        assert_eq!(env::predecessor_account_id(), owner_id, "Only the token owner can burn the token");

        self.tokens.remove(&token_id);
        self.token_to_owner.remove(&token_id);
        self.token_metadata.remove(&token_id);
        self.governance_rights.remove(&token_id);
    }

    pub fn revoke(&mut self, token_id: TokenId) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only the contract owner can revoke tokens");
        assert!(self.tokens.contains(&token_id), "Token does not exist");

        self.tokens.remove(&token_id);
        self.token_to_owner.remove(&token_id);
        self.token_metadata.remove(&token_id);
        self.governance_rights.remove(&token_id);
    }

    pub fn get_token_owner(&self, token_id: TokenId) -> Option<AccountId> {
        self.token_to_owner.get(&token_id)
    }

    pub fn get_token_metadata(&self, token_id: TokenId) -> Option<TokenMetadata> {
        self.token_metadata.get(&token_id)
    }

    pub fn get_governance_rights(&self, token_id: TokenId) -> Option<GovernanceRights> {
        self.governance_rights.get(&token_id)
    }

    pub fn update_governance_rights(&mut self, token_id: TokenId, new_rights: GovernanceRights) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only the contract owner can update governance rights");
        assert!(self.tokens.contains(&token_id), "Token does not exist");

        self.governance_rights.insert(&token_id, &new_rights);
    }

    pub fn add_awarded_title(&mut self, token_id: TokenId, title: String) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only the contract owner can add awarded titles");
        assert!(self.tokens.contains(&token_id), "Token does not exist");

        let mut metadata = self.token_metadata.get(&token_id).expect("Metadata not found");
        metadata.awarded_titles.push(title);
        self.token_metadata.insert(&token_id, &metadata);
    }
}