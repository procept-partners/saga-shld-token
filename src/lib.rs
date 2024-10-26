use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::store::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, require, AccountId, BorshStorageKey, NearToken, PanicOnDefault};
use near_sdk::serde_json;
use crate::serde_json::json;

#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKey {
    Tokens,
    TokenOwners,
    Proposals,
    ProposalVoters { proposal_id: u64 },
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct SHLDContract {
    tokens: LookupMap<AccountId, Token>,
    token_owners: UnorderedSet<AccountId>,
    proposals: UnorderedMap<u64, Proposal>,
    next_proposal_id: u64,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct SHLDContract {
    tokens: LookupMap<AccountId, Token>,          // Mapping from AccountId to Token data
    token_owners: UnorderedSet<AccountId>,        // Set of account IDs that own tokens
    proposals: UnorderedMap<u64, Proposal>,       // Mapping of proposal IDs to proposal details
    next_proposal_id: u64,                        // Tracks the next available ID for proposals
    members_registry: UnorderedSet<String>,       // Set to track unique cooperative member IDs by cooperative_id
    next_nft_number: u64,                         // Tracks the next NFT number in the overall series for uniqueness
    current_minting_round: u64,                   // Tracks the minting round for the SHLD token
    minting_order_in_round: u64,                  // Tracks the order of each token within the current minting round

}


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Token {
    owner_id: AccountId,
    metadata: TokenMetadata,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
// Replace the simplified TokenMetadata with this version in your code
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    ticker_title: String,                  // Fixed as "SHLD" for token type
    avatar_name: Option<String>,           // Unique display name for the token owner
    profile_description: Option<String>,   // Description specific to the token owner's profile
    governance_role: String,               // User's primary role in governance (e.g., "Member", "Admin")
    profile_image_url: Option<String>,     // URL or IPFS hash for profile image

    // Fields for Uniqueness and Verification
    cooperative_id: String,                // Unique cooperative identifier for each member
    did: Option<String>,                   // Optional decentralized identifier (DID) for identity management
    verification_status: String,           // Tracks verification status or attestation info
    minting_timestamp: u64,                // Timestamp of minting for record-keeping
    nft_number: u64,                       // Unique NFT identifier in the overall series
    minting_round: u64,                    // Identifier for the minting round or batch
    minting_order_in_round: u64,           // Order of the token within the current minting round
    unique_hash: String,                   // Unique hash identifier for the token
    
    // New Field
    member_titles: Vec<String>,            // Array of titles awarded to the member
}

// Add the following functions to the SHLDContract implementation in your code

// Update avatar_name
pub fn update_avatar_name(&mut self, account_id: AccountId, new_avatar_name: String) {
    let mut token = self.tokens.get(&account_id).expect("Token does not exist for this account");
    token.metadata.avatar_name = Some(new_avatar_name);
    self.tokens.insert(&account_id, &token);
}

// Update profile_description
pub fn update_profile_description(&mut self, account_id: AccountId, new_description: String) {
    let mut token = self.tokens.get(&account_id).expect("Token does not exist for this account");
    token.metadata.profile_description = Some(new_description);
    self.tokens.insert(&account_id, &token);
}

// Update profile_image_url
pub fn update_profile_image_url(&mut self, account_id: AccountId, new_image_url: String) {
    let mut token = self.tokens.get(&account_id).expect("Token does not exist for this account");
    token.metadata.profile_image_url = Some(new_image_url);
    self.tokens.insert(&account_id, &token);
}

// Update decentralized identifier (DID)
pub fn update_did(&mut self, account_id: AccountId, new_did: String) {
    let mut token = self.tokens.get(&account_id).expect("Token does not exist for this account");
    token.metadata.did = Some(new_did);
    self.tokens.insert(&account_id, &token);
}

// Add a new title to member_titles
pub fn add_member_title(&mut self, account_id: AccountId, new_title: String) {
    let mut token = self.tokens.get(&account_id).expect("Token does not exist for this account");
    token.metadata.member_titles.push(new_title);
    self.tokens.insert(&account_id, &token);
}

// Add this function to the SHLDContract implementation in your code

// Function to revoke an NFT (Only contract owner or treasury)
pub fn revoke_nft(&mut self, account_id: AccountId) {
    require!(env::predecessor_account_id() == env::current_account_id(), "Only the contract owner can revoke NFTs");

    let token = self.tokens.remove(&account_id).expect("Token does not exist for this account");
    self.token_owners.remove(&account_id);
    self.members_registry.remove(&token.metadata.cooperative_id);
}


#[derive(BorshDeserialize, BorshSerialize)]
//#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    id: u64,
    title: String,
    description: String,
    proposer: AccountId,
    votes_for: NearToken,
    votes_against: NearToken,
    //#[serde(skip)]
    voters: UnorderedSet<AccountId>,
    status: ProposalStatus,
}

impl Proposal {
    pub fn to_json_value(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "title": self.title,
            "description": self.description,
            "proposer": self.proposer,
            "votes_for": self.votes_for.as_near(),
            "votes_against": self.votes_against.as_near(),
            "status": self.status
        })
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
}

#[near_bindgen]
impl SHLDContract {
    #[init]
    pub fn new() -> Self {
        Self {
            tokens: LookupMap::new(StorageKey::Tokens),
            token_owners: UnorderedSet::new(StorageKey::TokenOwners),
            proposals: UnorderedMap::new(StorageKey::Proposals),
            next_proposal_id: 0,
        }
    }

    // Function to generate a unique hash for each NFT using cooperative ID and NFT number
    fn generate_unique_hash(&self, cooperative_id: &String, nft_number: u64) -> String {
        format!("{}-{}", cooperative_id, nft_number)
    }

    pub fn mint(&mut self, account_id: AccountId, metadata: TokenMetadata) {
        require!(!self.tokens.contains_key(&account_id), "Token already exists for this account");

        let token = Token {
            owner_id: account_id.clone(),
            metadata,
        };

        self.tokens.insert(account_id.clone(), token);
        self.token_owners.insert(account_id);
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

    pub fn create_proposal(&mut self, title: String, description: String) -> u64 {
        let account_id = env::predecessor_account_id();
        require!(self.is_token_owner(account_id.clone()), "Only SHLD holders can create proposals");
        
        let proposal_id = self.next_proposal_id;
        self.next_proposal_id += 1;

        let proposal = Proposal {
            id: proposal_id,
            title,
            description,
            proposer: account_id,
            votes_for: NearToken::from_near(0),
            votes_against: NearToken::from_near(0),
            voters: UnorderedSet::new(StorageKey::ProposalVoters { proposal_id }),
            status: ProposalStatus::Active,
        };

        self.proposals.insert(proposal_id, proposal);

        proposal_id
    }

    pub fn vote(&mut self, proposal_id: u64, vote: bool) {
        let account_id = env::predecessor_account_id();
        require!(self.is_token_owner(account_id.clone()), "Only SHLD holders can vote");
        

        if let Some(proposal) = self.proposals.get_mut(&proposal_id) {
            require!(proposal.status == ProposalStatus::Active, "Proposal is not active");
            require!(!proposal.voters.contains(&account_id), "Account has already voted");
    
            if vote {
                proposal.votes_for = proposal.votes_for.saturating_add(NearToken::from_near(1));
            } else {
                proposal.votes_against = proposal.votes_against.saturating_add(NearToken::from_near(1));
            }
    
            proposal.voters.insert(account_id);
    
            let total_votes = proposal.votes_for.as_near() + proposal.votes_against.as_near();
            if total_votes >= (self.token_owners.len() / 2 + 1) as u128 {
                proposal.status = if proposal.votes_for > proposal.votes_against {
                    ProposalStatus::Passed
                } else {
                    ProposalStatus::Rejected
                };
            }
        } else {
            env::panic_str("Proposal not found");
        }

        /*let mut proposal = self.proposals.get(&proposal_id).expect("Proposal not found").clone();
        require!(proposal.status == ProposalStatus::Active, "Proposal is not active");
        require!(!proposal.voters.contains(&account_id), "Account has already voted");

        if vote {
            proposal.votes_for = proposal.votes_for.saturating_add(NearToken::from_near(1));
        } else {
            proposal.votes_against = proposal.votes_against.saturating_add(NearToken::from_near(1));
        }

        proposal.voters.insert(account_id);

        let total_votes = proposal.votes_for.as_near() + proposal.votes_against.as_near();
        if total_votes >= (self.token_owners.len() / 2 + 1) as u128 {
            proposal.status = if proposal.votes_for > proposal.votes_against {
                ProposalStatus::Passed
            } else {
                ProposalStatus::Rejected    
            };
        }

        self.proposals.insert(proposal_id, proposal);*/
    }

    pub fn get_proposal(&self, proposal_id: u64) -> Option<serde_json::Value> {
        //self.proposals.get(&proposal_id)
        self.proposals.get(&proposal_id).map(|p| p.to_json_value())
        /*self.proposals.get(&proposal_id).map(|proposal| {
            Proposal {
                id: proposal.id,
                title: proposal.title.clone(),
                description: proposal.description.clone(),
                proposer: proposal.proposer.clone(),
                votes_for: proposal.votes_for,
                votes_against: proposal.votes_against,
                voters: UnorderedSet::new(StorageKey::ProposalVoters { 
                    proposal_id: proposal.id 
                }),
                status: proposal.status.clone(),
            }
        })*/
    }

    pub fn get_all_proposals(&self) -> Vec<serde_json::Value> {
        //self.proposals.values().collect()
        self.proposals.values().map(|p| p.to_json_value()).collect()
        /*self.proposals.values()
        .map(|proposal| Proposal {
            id: proposal.id,
            title: proposal.title.clone(),
            description: proposal.description.clone(),
            proposer: proposal.proposer.clone(),
            votes_for: proposal.votes_for,
            votes_against: proposal.votes_against,
            voters: UnorderedSet::new(StorageKey::ProposalVoters { 
                proposal_id: proposal.id 
            }),
            status: proposal.status.clone(),
        })
        .collect()*/
    }

    pub fn transfer(&mut self, _from: AccountId, _to: AccountId) {
        env::panic_str("SHLD tokens are non-transferable");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, VMContext};
    use serde_json::Value;

    fn get_context(predecessor_account_id: AccountId) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id(predecessor_account_id)
            .build()
    }

    fn setup_contract() -> (SHLDContract, AccountId) {
        let account_id = accounts(0);
        let context = get_context(account_id.clone());
        testing_env!(context);
        
        let contract = SHLDContract::new();
        (contract, account_id)
    }

    #[test]
    fn test_mint_token() {
        let (mut contract, account_id) = setup_contract();
        let metadata = TokenMetadata {
            title: Some("Test Token".to_string()),
            description: Some("Test Description".to_string()),
            governance_role: "Member".to_string(),
        };

        contract.mint(account_id.clone(), metadata.clone());

        assert!(contract.is_token_owner(account_id.clone()));
        assert_eq!(contract.token_metadata(account_id.clone()), Some(metadata));
    }

    #[test]
    #[should_panic(expected = "Token already exists for this account")]
    fn test_mint_token_already_exists() {
        let (mut contract, account_id) = setup_contract();
        let metadata = TokenMetadata {
            title: Some("Test Token".to_string()),
            description: Some("Test Description".to_string()),
            governance_role: "Member".to_string(),
        };

        contract.mint(account_id.clone(), metadata.clone());
        contract.mint(account_id, metadata); // Should panic
    }

    #[test]
    fn test_create_proposal() {
        let (mut contract, account_id) = setup_contract();
        let metadata = TokenMetadata {
            title: Some("Test Token".to_string()),
            description: Some("Test Description".to_string()),
            governance_role: "Member".to_string(),
        };

        contract.mint(account_id.clone(), metadata);

        let proposal_id = contract.create_proposal(
            "Test Proposal".to_string(),
            "Test Description".to_string(),
        );

        let proposal = contract.get_proposal(proposal_id).unwrap();
        //assert_eq!(proposal.title, "Test Proposal");
        //assert_eq!(proposal.description, "Test Description");
        //assert_eq!(proposal.proposer, account_id);
        //assert_eq!(proposal.status, ProposalStatus::Active);
        assert_eq!(proposal.get("title").and_then(Value::as_str).unwrap(), "Test Proposal");
        assert_eq!(proposal.get("description").and_then(Value::as_str).unwrap(), "Test Description");
        assert_eq!(proposal.get("proposer").and_then(Value::as_str).unwrap(), account_id.to_string());
        assert_eq!(proposal.get("status").and_then(Value::as_str).unwrap(), "Active");
    }

    #[test]
    #[should_panic(expected = "Only SHLD holders can create proposals")]
    fn test_create_proposal_non_token_holder() {
        let (mut contract, _) = setup_contract();
        let non_holder = accounts(1);
        testing_env!(get_context(non_holder));

        contract.create_proposal(
            "Test Proposal".to_string(),
            "Test Description".to_string(),
        );
    }

    #[test]
    fn test_vote_on_proposal() {
        let (mut contract, account_id) = setup_contract();
        // Mint a token for the first account
        let metadata = TokenMetadata {
            title: Some("Test Token".to_string()),
            description: Some("Test Description".to_string()),
            governance_role: "Member".to_string(),
        };
        contract.mint(account_id.clone(), metadata.clone());

        // Create a proposal
        let proposal_id = contract.create_proposal(
            "Test Proposal".to_string(),
            "Test Description".to_string(),
        );

        // Vote on the proposal
        contract.vote(proposal_id, true);

        let proposal = contract.get_proposal(proposal_id).unwrap();
        //assert_eq!(proposal.votes_for, NearToken::from_near(1));
        //assert_eq!(proposal.votes_against, NearToken::from_near(0));
        assert_eq!(proposal.get("votes_for").and_then(Value::as_u64).unwrap(), 1);
        assert_eq!(proposal.get("votes_against").and_then(Value::as_u64).unwrap(), 0);
    }

    /*#[test]
    #[should_panic(expected = "Account has already voted")]
    fn test_vote_twice() {
        let (mut contract, account_id) = setup_contract();
        let metadata = TokenMetadata {
            title: Some("Test Token".to_string()),
            description: Some("Test Description".to_string()),
            governance_role: "Member".to_string(),
        };
        contract.mint(account_id.clone(), metadata);

        let proposal_id = contract.create_proposal(
            "Test Proposal".to_string(),
            "Test Description".to_string(),
        );

        contract.vote(proposal_id, true);
        contract.vote(proposal_id, true); // Should panic
    }*/

    #[test]
    fn test_get_all_proposals() {
        let (mut contract, account_id) = setup_contract();
        let metadata = TokenMetadata {
            title: Some("Test Token".to_string()),
            description: Some("Test Description".to_string()),
            governance_role: "Member".to_string(),
        };
        contract.mint(account_id.clone(), metadata);

        // Create multiple proposals
        let proposal_id1 = contract.create_proposal(
            "Proposal 1".to_string(),
            "Description 1".to_string(),
        );
        let proposal_id2 = contract.create_proposal(
            "Proposal 2".to_string(),
            "Description 2".to_string(),
        );

        let proposals = contract.get_all_proposals();
        assert_eq!(proposals.len(), 2);
        //assert_eq!(proposals[0].id, proposal_id1);
        //assert_eq!(proposals[1].id, proposal_id2);
        assert_eq!(proposals[0].get("id").and_then(Value::as_u64).unwrap(), proposal_id1);
        assert_eq!(proposals[1].get("id").and_then(Value::as_u64).unwrap(), proposal_id2);
    }

    #[test]
    #[should_panic(expected = "SHLD tokens are non-transferable")]
    fn test_transfer_not_allowed() {
        let (mut contract, account_id) = setup_contract();
        contract.transfer(account_id, accounts(1));
    }
}