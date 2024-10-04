use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault};

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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Token {
    owner_id: AccountId,
    metadata: TokenMetadata,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    title: Option<String>,
    description: Option<String>,
    governance_role: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    id: u64,
    title: String,
    description: String,
    proposer: AccountId,
    votes_for: Balance,
    votes_against: Balance,
    voters: UnorderedSet<AccountId>,
    status: ProposalStatus,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
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

    pub fn mint(&mut self, account_id: AccountId, metadata: TokenMetadata) {
        assert!(!self.tokens.contains_key(&account_id), "Token already exists for this account");
        
        let token = Token {
            owner_id: account_id.clone(),
            metadata,
        };
        
        self.tokens.insert(&account_id, &token);
        self.token_owners.insert(&account_id);
    }

    pub fn token_metadata(&self, account_id: AccountId) -> Option<TokenMetadata> {
        self.tokens.get(&account_id).map(|token| token.metadata)
    }

    pub fn is_token_owner(&self, account_id: AccountId) -> bool {
        self.token_owners.contains(&account_id)
    }

    pub fn governance_role(&self, account_id: AccountId) -> Option<String> {
        self.tokens.get(&account_id).map(|token| token.metadata.governance_role)
    }

    pub fn create_proposal(&mut self, title: String, description: String) -> u64 {
        let account_id = env::predecessor_account_id();
        assert!(self.is_token_owner(account_id.clone()), "Only SHLD holders can create proposals");
        
        let proposal_id = self.next_proposal_id;
        self.next_proposal_id += 1;

        let proposal = Proposal {
            id: proposal_id,
            title,
            description,
            proposer: account_id,
            votes_for: 0,
            votes_against: 0,
            voters: UnorderedSet::new(StorageKey::ProposalVoters { proposal_id }),
            status: ProposalStatus::Active,
        };

        self.proposals.insert(&proposal_id, &proposal);

        proposal_id
    }

    pub fn vote(&mut self, proposal_id: u64, vote: bool) {
        let account_id = env::predecessor_account_id();
        assert!(self.is_token_owner(account_id.clone()), "Only SHLD holders can vote");
        
        let mut proposal = self.proposals.get(&proposal_id).expect("Proposal not found");
        assert!(proposal.status == ProposalStatus::Active, "Proposal is not active");
        assert!(!proposal.voters.contains(&account_id), "Account has already voted");

        if vote {
            proposal.votes_for += 1;
        } else {
            proposal.votes_against += 1;
        }

        proposal.voters.insert(&account_id);

        let total_votes = proposal.votes_for + proposal.votes_against;
        if total_votes >= self.token_owners.len() / 2 + 1 {
            if proposal.votes_for > proposal.votes_against {
                proposal.status = ProposalStatus::Passed;
            } else {
                proposal.status = ProposalStatus::Rejected;
            }
        }

        self.proposals.insert(&proposal_id, &proposal);
    }

    pub fn get_proposal(&self, proposal_id: u64) -> Option<Proposal> {
        self.proposals.get(&proposal_id)
    }

    pub fn get_all_proposals(&self) -> Vec<Proposal> {
        self.proposals.values().collect()
    }

    pub fn transfer(&mut self, _from: AccountId, _to: AccountId) {
        env::panic_str("SHLD tokens are non-transferable");
    }
}