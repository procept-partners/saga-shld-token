/*use serde_json::json;

#[tokio::test]
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

[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    fn get_context(predecessor_account_id: AccountId) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id(predecessor_account_id)
            .build()
    }

    #[test]
    fn test_create_proposal_and_vote() {
        let mut contract = SHLDContract::new();
        let account_id1 = AccountId::new_unchecked("alice.near".to_string());
        let account_id2 = AccountId::new_unchecked("bob.near".to_string());
        
        let metadata = TokenMetadata {
            title: Some("SHLD Token".to_string()),
            description: Some("Governance Token".to_string()),
            governance_role: "Voter".to_string(),
        };
        
        contract.mint(account_id1.clone(), metadata.clone());
        contract.mint(account_id2.clone(), metadata);

        testing_env!(get_context(account_id1.clone()));
        let proposal_id = contract.create_proposal("Test Proposal".to_string(), "This is a test proposal".to_string());

        testing_env!(get_context(account_id1.clone()));
        contract.vote(proposal_id, true);

        testing_env!(get_context(account_id2.clone()));
        contract.vote(proposal_id, false);

        let proposal = contract.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.votes_for, 1);
        assert_eq!(proposal.votes_against, 1);
        assert_eq!(proposal.status, ProposalStatus::Active);
    }
}