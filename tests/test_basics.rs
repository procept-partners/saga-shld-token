use serde_json::json;
use near_sdk::test_utils::{accounts, VMContextBuilder};
use near_sdk::{testing_env, VMContext, AccountId, NearToken};
use SHLD_Token::SHLDContract;
use SHLD_Token::TokenMetadata;
use SHLD_Token::ProposalStatus;

//use crate::{SHLDContract, TokenMetadata, ProposalStatus};

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
fn test_new() {
    let (contract, _) = setup_contract();
    assert_eq!(contract.get_all_proposals().len(), 0);
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
    assert_eq!(contract.governance_role(account_id), Some("Member".to_string()));
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
    assert_eq!(proposal.title, "Test Proposal");
    assert_eq!(proposal.description, "Test Description");
    assert_eq!(proposal.proposer, account_id);
    assert_eq!(proposal.status, ProposalStatus::Active);
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
    let metadata = TokenMetadata {
        title: Some("Test Token".to_string()),
        description: Some("Test Description".to_string()),
        governance_role: "Member".to_string(),
    };
    contract.mint(account_id.clone(), metadata.clone());

    let proposal_id = contract.create_proposal(
        "Test Proposal".to_string(),
        "Test Description".to_string(),
    );

    contract.vote(proposal_id, true);

    let proposal = contract.get_proposal(proposal_id).unwrap();
    assert_eq!(proposal.votes_for, NearToken::from_near(1));
    assert_eq!(proposal.votes_against, NearToken::from_near(0));
    assert_eq!(proposal.status, ProposalStatus::Active); // Should still be active after one vote
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
fn test_proposal_passed() {
    let (mut contract, account_id) = setup_contract();
    let metadata = TokenMetadata {
        title: Some("Test Token".to_string()),
        description: Some("Test Description".to_string()),
        governance_role: "Member".to_string(),
    };
    contract.mint(account_id.clone(), metadata.clone());

    // Mint tokens for two more accounts to have a total of 3 token holders
    contract.mint(accounts(1), metadata.clone());
    contract.mint(accounts(2), metadata);

    let proposal_id = contract.create_proposal(
        "Test Proposal".to_string(),
        "Test Description".to_string(),
    );

    // Vote with all three accounts
    contract.vote(proposal_id, true);
    testing_env!(get_context(accounts(1)));
    contract.vote(proposal_id, true);
    testing_env!(get_context(accounts(2)));
    contract.vote(proposal_id, false);

    let proposal = contract.get_proposal(proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Passed);
    assert_eq!(proposal.votes_for, NearToken::from_near(2));
    assert_eq!(proposal.votes_against, NearToken::from_near(1));
}

#[test]
fn test_get_all_proposals() {
    let (mut contract, account_id) = setup_contract();
    let metadata = TokenMetadata {
        title: Some("Test Token".to_string()),
        description: Some("Test Description".to_string()),
        governance_role: "Member".to_string(),
    };
    contract.mint(account_id.clone(), metadata);

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
    assert_eq!(proposals[0].id, proposal_id1);
    assert_eq!(proposals[1].id, proposal_id2);
}

#[test]
#[should_panic(expected = "SHLD tokens are non-transferable")]
fn test_transfer_not_allowed() {
    let (mut contract, account_id) = setup_contract();
    contract.transfer(account_id, accounts(1));
}

/*#[tokio::test]
async fn test_contract_is_operational() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = near_workspaces::compile_project("./").await?;

    let contract = sandbox.dev_deploy(&contract_wasm).await?;

    let user_account = sandbox.dev_create_account().await?;

    let outcome = user_account
        .call(contract.id(), "set_greeting")
        .args_json(json!({"greeting": "Hello World!"}))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let user_message_outcome = contract.view("get_greeting").args_json(json!({})).await?;
    assert_eq!(user_message_outcome.json::<String>()?, "Hello World!");

    Ok(())
}*/