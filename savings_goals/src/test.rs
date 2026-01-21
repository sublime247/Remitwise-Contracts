#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Ledger, Env, String};

#[test]
fn test_create_goal_unique_ids() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SavingsGoalContract);
    let client = SavingsGoalContractClient::new(&env, &contract_id);

    client.init();

    let name1 = String::from_str(&env, "Goal 1");
    let name2 = String::from_str(&env, "Goal 2");

    let id1 = client.create_goal(&name1, &1000, &1735689600); // Dec 2024
    let id2 = client.create_goal(&name2, &2000, &1735689600);

    assert_ne!(id1, id2);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
fn test_add_to_goal_increments() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SavingsGoalContract);
    let client = SavingsGoalContractClient::new(&env, &contract_id);

    client.init();
    let id = client.create_goal(&String::from_str(&env, "Save"), &1000, &2000000000);

    let new_balance = client.add_to_goal(&id, &500);
    assert_eq!(new_balance, 500);

    let goal = client.get_goal(&id).unwrap();
    assert_eq!(goal.current_amount, 500);
}

#[test]
fn test_add_to_non_existent_goal() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SavingsGoalContract);
    let client = SavingsGoalContractClient::new(&env, &contract_id);

    client.init();
    let result = client.add_to_goal(&99, &500);
    assert_eq!(result, -1);
}

#[test]
fn test_get_goal_retrieval() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SavingsGoalContract);
    let client = SavingsGoalContractClient::new(&env, &contract_id);

    client.init();
    let name = String::from_str(&env, "Car");
    let id = client.create_goal(&name, &5000, &2000000000);

    let goal = client.get_goal(&id).unwrap();
    assert_eq!(goal.name, name);
    assert_eq!(goal.target_amount, 5000);
}

#[test]
fn test_get_all_goals() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SavingsGoalContract);
    let client = SavingsGoalContractClient::new(&env, &contract_id);

    client.init();
    client.create_goal(&String::from_str(&env, "A"), &100, &2000000000);
    client.create_goal(&String::from_str(&env, "B"), &200, &2000000000);

    let all_goals = client.get_all_goals();
    assert_eq!(all_goals.len(), 2);
}

#[test]
fn test_is_goal_completed() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SavingsGoalContract);
    let client = SavingsGoalContractClient::new(&env, &contract_id);

    client.init();
    let id = client.create_goal(&String::from_str(&env, "Trip"), &1000, &2000000000);

    // Test not completed
    assert_eq!(client.is_goal_completed(&id), false);

    // Test completed (exactly target)
    client.add_to_goal(&id, &1000);
    assert_eq!(client.is_goal_completed(&id), true);

    // Test completed (over target)
    client.add_to_goal(&id, &1);
    assert_eq!(client.is_goal_completed(&id), true);
}

#[test]
fn test_edge_cases_large_amounts() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SavingsGoalContract);
    let client = SavingsGoalContractClient::new(&env, &contract_id);

    client.init();
    let id = client.create_goal(&String::from_str(&env, "Max"), &i128::MAX, &2000000000);

    // Test large add (should work if within i128)
    client.add_to_goal(&id, &(i128::MAX - 100));
    let goal = client.get_goal(&id).unwrap();
    assert_eq!(goal.current_amount, i128::MAX - 100);
}

#[test]
#[should_panic(expected = "Target amount must be positive")]
fn test_zero_amount_fails() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SavingsGoalContract);
    let client = SavingsGoalContractClient::new(&env, &contract_id);

    client.init();
    client.create_goal(&String::from_str(&env, "Fail"), &0, &2000000000);
}

#[test]
fn test_multiple_goals_management() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SavingsGoalContract);
    let client = SavingsGoalContractClient::new(&env, &contract_id);

    client.init();
    let id1 = client.create_goal(&String::from_str(&env, "G1"), &1000, &2000000000);
    let id2 = client.create_goal(&String::from_str(&env, "G2"), &2000, &2000000000);

    client.add_to_goal(&id1, &500);
    client.add_to_goal(&id2, &1500);

    let g1 = client.get_goal(&id1).unwrap();
    let g2 = client.get_goal(&id2).unwrap();

    assert_eq!(g1.current_amount, 500);
    assert_eq!(g2.current_amount, 1500);
}
