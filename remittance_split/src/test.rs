#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, vec, Env};

#[test]
fn test_initialize_split() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    
    // Test valid split
    let result = client.initialize_split(&50, &30, &15, &5);
    assert!(result);
    
    // Test invalid split (doesn't sum to 100)
    let result = client.initialize_split(&50, &30, &15, &10);
    assert!(!result);
}

#[test]
fn test_get_split() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    
    // Should return default split if not initialized
    let split = client.get_split();
    assert_eq!(split.get(0).unwrap(), 50);
    assert_eq!(split.get(1).unwrap(), 30);
    assert_eq!(split.get(2).unwrap(), 15);
    assert_eq!(split.get(3).unwrap(), 5);
    
    // Initialize and test
    client.initialize_split(&60, &25, &10, &5);
    let split = client.get_split();
    assert_eq!(split.get(0).unwrap(), 60);
    assert_eq!(split.get(1).unwrap(), 25);
    assert_eq!(split.get(2).unwrap(), 10);
    assert_eq!(split.get(3).unwrap(), 5);
}

#[test]
fn test_calculate_split() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    
    // Initialize split: 50% spending, 30% savings, 15% bills, 5% insurance
    client.initialize_split(&50, &30, &15, &5);
    
    // Test with $300 (30000 in smallest unit, assuming 2 decimals)
    let total = 30000i128;
    let amounts = client.calculate_split(&total);
    
    assert_eq!(amounts.get(0).unwrap(), 15000); // 50% spending
    assert_eq!(amounts.get(1).unwrap(), 9000);  // 30% savings
    assert_eq!(amounts.get(2).unwrap(), 4500);  // 15% bills
    assert_eq!(amounts.get(3).unwrap(), 1500);  // 5% insurance
}

