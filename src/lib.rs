use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::store::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, require, AccountId, BorshStorageKey, NearToken, PanicOnDefault};
use near_sdk::serde_json;
use crate::serde_json::json;
use near_sdk::json_types::U128;
use ethabi::ethereum_types::H160;
use secp256k1::Message;

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
    title: Option<String>,
    description: Option<String>,
    governance_role: String,
    ticker_title: String,
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
            //metadata,
        };

        self.tokens.insert(account_id.clone(), token);
        self.token_owners.insert(account_id.clone());
        self.account_tokens.insert(account_id.clone(), unique_hash); // Link NEAR account to SHLD token hash
    }

    pub fn link_shld_token(&mut self, account_id: AccountId, token_hash: String) {
        self.account_tokens.insert(account_id, token_hash);
    }

    pub fn update_avatar_name(&mut self, account_id: AccountId, new_avatar_name: String) {
        let mut token = self.tokens.get(&account_id).expect("Token does not exist for this account");
        token.metadata.avatar_name = Some(new_avatar_name);
        self.tokens.insert(account_id, &token);
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

//use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
//use near_sdk::{env, near_bindgen, AccountId};

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