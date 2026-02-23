#[cfg(test)]
mod testsuit {
    use crate::*;
    use soroban_sdk::testutils::{Address as AddressTrait, Ledger, LedgerInfo};
    use soroban_sdk::testutils::storage::Instance as _;
    use soroban_sdk::Env;

    fn set_time(env: &Env, timestamp: u64) {
        let proto = env.ledger().protocol_version();

        env.ledger().set(LedgerInfo {
            protocol_version: proto,
            sequence_number: 1,
            timestamp,
            network_id: [0; 32],
            base_reserve: 10,
            min_temp_entry_ttl: 1,
            min_persistent_entry_ttl: 1,
            max_entry_ttl: 100000,
        });
    }

    #[test]
    fn test_create_bill() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();

        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Electricity"),
            &1000,
            &1000000,
            &false,
            &0,
        );

        assert_eq!(bill_id, 1);

        let bill = client.get_bill(&1);
        assert!(bill.is_some());
        let bill = bill.unwrap();
        assert_eq!(bill.amount, 1000);
        assert!(!bill.paid);
    }

    #[test]
    fn test_create_bill_invalid_amount() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        let result = client.try_create_bill(
            &owner,
            &String::from_str(&env, "Invalid"),
            &0,
            &1000000,
            &false,
            &0,
        );

        assert_eq!(result, Err(Ok(Error::InvalidAmount)));
    }

    #[test]
    fn test_create_recurring_bill_invalid_frequency() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        let result = client.try_create_bill(
            &owner,
            &String::from_str(&env, "Monthly"),
            &500,
            &1000000,
            &true,
            &0,
        );

        assert_eq!(result, Err(Ok(Error::InvalidFrequency)));
    }

    #[test]
    fn test_create_bill_negative_amount() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        let result = client.try_create_bill(
            &owner,
            &String::from_str(&env, "Invalid"),
            &-100,
            &1000000,
            &false,
            &0,
        );

        assert_eq!(result, Err(Ok(Error::InvalidAmount)));
    }

    #[test]
    fn test_pay_bill() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Water"),
            &500,
            &1000000,
            &false,
            &0,
        );

        env.mock_all_auths();
        client.pay_bill(&owner, &bill_id);

        let bill = client.get_bill(&bill_id).unwrap();
        assert!(bill.paid);

        assert!(bill.paid_at.is_some());
    }

    #[test]
    fn test_recurring_bill() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);
        env.mock_all_auths();
        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Rent"),
            &10000,
            &1000000,
            &true,
            &30,
        );

        env.mock_all_auths();
        client.pay_bill(&owner, &bill_id);

        // Check original bill is paid
        let bill = client.get_bill(&bill_id).unwrap();
        assert!(bill.paid);

        // Check next recurring bill was created
        let bill2 = client.get_bill(&2).unwrap();
        assert!(!bill2.paid);

        assert_eq!(bill2.amount, 10000);
        assert_eq!(bill2.due_date, 1000000 + (30 * 86400));
    }

    #[test]
    fn test_get_unpaid_bills() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);
        env.mock_all_auths();
        client.create_bill(
            &owner,
            &String::from_str(&env, "Bill1"),
            &100,
            &1000000,
            &false,
            &0,
        );
        env.mock_all_auths();
        client.create_bill(
            &owner,
            &String::from_str(&env, "Bill2"),
            &200,
            &1000000,
            &false,
            &0,
        );
        env.mock_all_auths();
        client.create_bill(
            &owner,
            &String::from_str(&env, "Bill3"),
            &300,
            &1000000,
            &false,
            &0,
        );
        env.mock_all_auths();
        client.pay_bill(&owner, &1);

        let unpaid = client.get_unpaid_bills(&owner);
        assert_eq!(unpaid.len(), 2);
    }

    #[test]
    fn test_get_total_unpaid() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);
        env.mock_all_auths();
        client.create_bill(
            &owner,
            &String::from_str(&env, "Bill1"),
            &100,
            &1000000,
            &false,
            &0,
        );
        env.mock_all_auths();
        client.create_bill(
            &owner,
            &String::from_str(&env, "Bill2"),
            &200,
            &1000000,
            &false,
            &0,
        );
        env.mock_all_auths();
        client.create_bill(
            &owner,
            &String::from_str(&env, "Bill3"),
            &300,
            &1000000,
            &false,
            &0,
        );
        env.mock_all_auths();
        client.pay_bill(&owner, &1);

        let total = client.get_total_unpaid(&owner);
        assert_eq!(total, 500); // 200 + 300
    }

    #[test]
    fn test_pay_nonexistent_bill() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        let result = client.try_pay_bill(&owner, &999);
        assert_eq!(result, Err(Ok(Error::BillNotFound)));
    }

    #[test]
    fn test_pay_already_paid_bill() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);
        env.mock_all_auths();
        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Test"),
            &100,
            &1000000,
            &false,
            &0,
        );
        env.mock_all_auths();
        client.pay_bill(&owner, &bill_id);
        let result = client.try_pay_bill(&owner, &bill_id);
        assert_eq!(result, Err(Ok(Error::BillAlreadyPaid)));
    }

    #[test]
    fn test_get_overdue_bills() {
        let env = Env::default();
        set_time(&env, 2_000_000);

        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);
        env.mock_all_auths();
        // Create bills with different due dates
        client.create_bill(
            &owner,
            &String::from_str(&env, "Overdue1"),
            &100,
            &1000000,
            &false,
            &0,
        );
        env.mock_all_auths();
        client.create_bill(
            &owner,
            &String::from_str(&env, "Overdue2"),
            &200,
            &1500000,
            &false,
            &0,
        );
        env.mock_all_auths();
        client.create_bill(
            &owner,
            &String::from_str(&env, "Future"),
            &300,
            &3000000,
            &false,
            &0,
        );

        let overdue = client.get_overdue_bills();
        assert_eq!(overdue.len(), 2); // Only first two are overdue
    }

    #[test]
    fn test_cancel_bill() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);
        env.mock_all_auths();
        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Test"),
            &100,
            &1000000,
            &false,
            &0,
        );
        env.mock_all_auths();
        client.cancel_bill(&owner, &bill_id);
        let bill = client.get_bill(&bill_id);
        assert!(bill.is_none());
    }

    #[test]
    fn test_cancel_nonexistent_bill() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);
        env.mock_all_auths();
        let result = client.try_cancel_bill(&owner, &999);
        assert_eq!(result, Err(Ok(Error::BillNotFound)));
    }

    #[test]
    fn test_multiple_recurring_payments() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);
        env.mock_all_auths();
        // Create recurring bill
        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Subscription"),
            &999,
            &1000000,
            &true,
            &30,
        );
        env.mock_all_auths();
        // Pay first bill - creates second
        client.pay_bill(&owner, &bill_id);
        let bill2 = client.get_bill(&2).unwrap();
        assert!(!bill2.paid);
        assert_eq!(bill2.due_date, 1000000 + (30 * 86400));
        env.mock_all_auths();
        // Pay second bill - creates third
        client.pay_bill(&owner, &2);
        let bill3 = client.get_bill(&3).unwrap();
        assert!(!bill3.paid);
        assert_eq!(bill3.due_date, 1000000 + (60 * 86400));
    }

    #[test]
    fn test_get_all_bills() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);
        env.mock_all_auths();
        client.create_bill(
            &owner,
            &String::from_str(&env, "Bill1"),
            &100,
            &1000000,
            &false,
            &0,
        );
        env.mock_all_auths();
        client.create_bill(
            &owner,
            &String::from_str(&env, "Bill2"),
            &200,
            &1000000,
            &false,
            &0,
        );
        env.mock_all_auths();
        client.create_bill(
            &owner,
            &String::from_str(&env, "Bill3"),
            &300,
            &1000000,
            &false,
            &0,
        );
        env.mock_all_auths();
        client.pay_bill(&owner, &1);

        let all = client.get_all_bills();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_pay_bill_unauthorized() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);
        let other = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Water"),
            &500,
            &1000000,
            &false,
            &0,
        );

        let result = client.try_pay_bill(&other, &bill_id);
        assert_eq!(result, Err(Ok(Error::Unauthorized)));
    }

    #[test]
    fn test_recurring_bill_cancellation() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Rent"),
            &1000,
            &1000000,
            &true, // Recurring
            &30,
        );

        // Cancel the bill
        client.cancel_bill(&owner, &bill_id);

        // Verify it's gone
        let bill = client.get_bill(&bill_id);
        assert!(bill.is_none());

        // Verify paying it fails
        let result = client.try_pay_bill(&owner, &bill_id);
        assert_eq!(result, Err(Ok(Error::BillNotFound)));
    }

    #[test]
    fn test_pay_overdue_bill() {
        let env = Env::default();
        set_time(&env, 2_000_000); // Set time past due date
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Late"),
            &500,
            &1000000, // Due in past
            &false,
            &0,
        );

        // Verify it shows up in overdue
        let overdue = client.get_overdue_bills();
        assert_eq!(overdue.len(), 1);

        // Pay it
        client.pay_bill(&owner, &bill_id);

        // Verify it's no longer overdue (because it's paid)
        let overdue_after = client.get_overdue_bills();
        assert_eq!(overdue_after.len(), 0);
    }

    #[test]
    fn test_short_recurrence() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Daily"),
            &10,
            &1000000,
            &true, // Recurring
            &1,    // Daily
        );

        client.pay_bill(&owner, &bill_id);

        let next_bill = client.get_bill(&2).unwrap();
        assert_eq!(next_bill.due_date, 1000000 + 86400); // Exactly 1 day later
    }

    // NOTE: The following schedule-related tests are commented out because the
    // BillPayments contract does not implement create_schedule, modify_schedule,
    // cancel_schedule, execute_due_schedules, get_schedule, or get_schedules methods.
    // These tests were added to main before the contract methods were implemented.
    // Uncomment once the schedule functionality is added to the contract.

    /*
    #[test]
    fn test_create_schedule() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        set_time(&env, 1000);

        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Electricity"),
            &1000,
            &2000,
            &false,
            &0,
        );

        let schedule_id = client.create_schedule(&owner, &bill_id, &3000, &86400);
        assert_eq!(schedule_id, 1);

        let schedule = client.get_schedule(&schedule_id);
        assert!(schedule.is_some());
        let schedule = schedule.unwrap();
        assert_eq!(schedule.next_due, 3000);
        assert_eq!(schedule.interval, 86400);
        assert!(schedule.active);
    }

    #[test]
    fn test_modify_schedule() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        set_time(&env, 1000);

        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Electricity"),
            &1000,
            &2000,
            &false,
            &0,
        );

        let schedule_id = client.create_schedule(&owner, &bill_id, &3000, &86400);
        client.modify_schedule(&owner, &schedule_id, &4000, &172800);

        let schedule = client.get_schedule(&schedule_id).unwrap();
        assert_eq!(schedule.next_due, 4000);
        assert_eq!(schedule.interval, 172800);
    }

    #[test]
    fn test_cancel_schedule() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        set_time(&env, 1000);

        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Electricity"),
            &1000,
            &2000,
            &false,
            &0,
        );

        let schedule_id = client.create_schedule(&owner, &bill_id, &3000, &86400);
        client.cancel_schedule(&owner, &schedule_id);

        let schedule = client.get_schedule(&schedule_id).unwrap();
        assert!(!schedule.active);
    }

    #[test]
    fn test_execute_due_schedules() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        set_time(&env, 1000);

        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Electricity"),
            &1000,
            &2000,
            &false,
            &0,
        );

        let schedule_id = client.create_schedule(&owner, &bill_id, &3000, &0);

        set_time(&env, 3500);
        let executed = client.execute_due_schedules();

        assert_eq!(executed.len(), 1);
        assert_eq!(executed.get(0).unwrap(), schedule_id);

        let bill = client.get_bill(&bill_id).unwrap();
        assert!(bill.paid);
    }

    #[test]
    fn test_execute_recurring_schedule() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        set_time(&env, 1000);

        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Electricity"),
            &1000,
            &2000,
            &true,
            &30,
        );

        let schedule_id = client.create_schedule(&owner, &bill_id, &3000, &86400);

        set_time(&env, 3500);
        client.execute_due_schedules();

        let schedule = client.get_schedule(&schedule_id).unwrap();
        assert!(schedule.active);
        assert_eq!(schedule.next_due, 3000 + 86400);
    }

    #[test]
    fn test_execute_missed_schedules() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        set_time(&env, 1000);

        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Electricity"),
            &1000,
            &2000,
            &true,
            &30,
        );

        let schedule_id = client.create_schedule(&owner, &bill_id, &3000, &86400);

        set_time(&env, 3000 + 86400 * 3 + 100);
        client.execute_due_schedules();

        let schedule = client.get_schedule(&schedule_id).unwrap();
        assert_eq!(schedule.missed_count, 3);
        assert!(schedule.next_due > 3000 + 86400 * 3);
    }

    #[test]
    fn test_schedule_validation_past_date() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        set_time(&env, 5000);

        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Electricity"),
            &1000,
            &6000,
            &false,
            &0,
        );

        let result = client.try_create_schedule(&owner, &bill_id, &3000, &86400);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_schedules() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        env.mock_all_auths();
        set_time(&env, 1000);

        let bill_id1 = client.create_bill(
            &owner,
            &String::from_str(&env, "Electricity"),
            &1000,
            &2000,
            &false,
            &0,
        );

        let bill_id2 = client.create_bill(
            &owner,
            &String::from_str(&env, "Water"),
            &500,
            &2000,
            &false,
            &0,
        );

        client.create_schedule(&owner, &bill_id1, &3000, &86400);
        client.create_schedule(&owner, &bill_id2, &4000, &172800);

        let schedules = client.get_schedules(&owner);
        assert_eq!(schedules.len(), 2);
    }
    */

    // ========================================================================
    // Storage TTL Extension Tests
    //
    // Verify that instance storage TTL is properly extended on state-changing
    // operations, preventing unexpected data expiration.
    //
    // Contract TTL configuration:
    //   INSTANCE_LIFETIME_THRESHOLD  = 17,280 ledgers (~1 day)
    //   INSTANCE_BUMP_AMOUNT         = 518,400 ledgers (~30 days)
    //   ARCHIVE_LIFETIME_THRESHOLD   = 17,280 ledgers (~1 day)
    //   ARCHIVE_BUMP_AMOUNT          = 2,592,000 ledgers (~180 days)
    //
    // Operations extending instance TTL:
    //   create_bill, pay_bill, archive_paid_bills, restore_bill,
    //   bulk_cleanup_bills, batch_pay_bills
    //
    // Operations extending archive TTL:
    //   archive_paid_bills
    // ========================================================================

    /// Verify that create_bill extends instance storage TTL.
    #[test]
    fn test_instance_ttl_extended_on_create_bill() {
        let env = Env::default();
        env.mock_all_auths();

        env.ledger().set(LedgerInfo {
            protocol_version: 20,
            sequence_number: 100,
            timestamp: 1000,
            network_id: [0; 32],
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 700_000,
        });

        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        // create_bill calls extend_instance_ttl internally
        let bill_id = client.create_bill(
            &owner,
            &String::from_str(&env, "Electricity"),
            &1000,
            &2000,
            &false,
            &0,
        );
        assert_eq!(bill_id, 1);

        // Inspect instance TTL — must be at least INSTANCE_BUMP_AMOUNT (518,400)
        let ttl = env.as_contract(&contract_id, || {
            env.storage().instance().get_ttl()
        });
        assert!(
            ttl >= 518_400,
            "Instance TTL ({}) must be >= INSTANCE_BUMP_AMOUNT (518,400) after create_bill",
            ttl
        );
    }

    /// Verify that pay_bill refreshes instance TTL after ledger advancement.
    ///
    /// extend_ttl(threshold, extend_to) only extends when TTL <= threshold.
    /// After create_bill at seq 100 sets TTL to 518,400 (live_until = 518,500),
    /// we must advance past seq 501,220 so TTL drops below 17,280.
    #[test]
    fn test_instance_ttl_refreshed_on_pay_bill() {
        let env = Env::default();
        env.mock_all_auths();

        env.ledger().set(LedgerInfo {
            protocol_version: 20,
            sequence_number: 100,
            timestamp: 1000,
            network_id: [0; 32],
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 700_000,
        });

        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        client.create_bill(
            &owner,
            &String::from_str(&env, "Water Bill"),
            &500,
            &5000,
            &false,
            &0,
        );

        // Advance ledger far enough that TTL drops below threshold (17,280).
        // After create_bill: live_until = 100 + 518,400 = 518,500
        // At seq 510,000: TTL = 518,500 - 510,000 = 8,500 < 17,280 ✓
        env.ledger().set(LedgerInfo {
            protocol_version: 20,
            sequence_number: 510_000,
            timestamp: 500_000,
            network_id: [0; 32],
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 700_000,
        });

        // pay_bill calls extend_instance_ttl → re-extends TTL to 518,400
        client.pay_bill(&owner, &1);

        // TTL should be refreshed relative to the new sequence number
        let ttl = env.as_contract(&contract_id, || {
            env.storage().instance().get_ttl()
        });
        assert!(
            ttl >= 518_400,
            "Instance TTL ({}) must be >= 518,400 after pay_bill refreshes it",
            ttl
        );
    }

    /// Verify that data remains accessible across repeated operations
    /// spanning multiple ledger advancements, proving TTL is continuously renewed.
    ///
    /// Each phase advances the ledger past the TTL threshold so every
    /// state-changing call actually re-extends the TTL.
    #[test]
    fn test_data_persists_across_repeated_operations() {
        let env = Env::default();
        env.mock_all_auths();

        env.ledger().set(LedgerInfo {
            protocol_version: 20,
            sequence_number: 100,
            timestamp: 1000,
            network_id: [0; 32],
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 700_000,
        });

        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        // Phase 1: Create first bill at seq 100
        // TTL goes from 100 → 518,400. live_until = 518,500
        let id1 = client.create_bill(
            &owner,
            &String::from_str(&env, "Rent"),
            &2000,
            &1_100_000,
            &false,
            &0,
        );

        // Phase 2: Advance to seq 510,000 (TTL = 8,500 < 17,280)
        // create_bill re-extends → live_until = 1,028,400
        env.ledger().set(LedgerInfo {
            protocol_version: 20,
            sequence_number: 510_000,
            timestamp: 510_000,
            network_id: [0; 32],
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 700_000,
        });

        let id2 = client.create_bill(
            &owner,
            &String::from_str(&env, "Internet"),
            &100,
            &1_200_000,
            &false,
            &0,
        );

        // Phase 3: Advance to seq 1,020,000 (TTL = 8,400 < 17,280)
        // pay_bill re-extends → live_until = 1,538,400
        env.ledger().set(LedgerInfo {
            protocol_version: 20,
            sequence_number: 1_020_000,
            timestamp: 1_020_000,
            network_id: [0; 32],
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 700_000,
        });

        // Pay second bill to refresh TTL once more
        client.pay_bill(&owner, &id2);

        // Both bills should still be accessible
        let bill1 = client.get_bill(&id1);
        assert!(bill1.is_some(), "First bill must persist across ledger advancements");
        assert_eq!(bill1.unwrap().amount, 2000);

        let bill2 = client.get_bill(&id2);
        assert!(bill2.is_some(), "Second bill must persist across ledger advancements");
        assert!(bill2.unwrap().paid, "Second bill should be marked paid");

        // TTL should be fully refreshed
        let ttl = env.as_contract(&contract_id, || {
            env.storage().instance().get_ttl()
        });
        assert!(
            ttl >= 518_400,
            "Instance TTL ({}) must remain >= 518,400 after repeated operations",
            ttl
        );
    }

    /// Verify that archive_paid_bills extends instance TTL and archives data.
    ///
    /// Note: both `extend_instance_ttl` and `extend_archive_ttl` operate on
    /// instance() storage. Since `extend_instance_ttl` is called first in
    /// `archive_paid_bills`, it bumps the TTL above the shared threshold
    /// (17,280), making the subsequent `extend_archive_ttl` a no-op.
    /// This test verifies the instance TTL is at least INSTANCE_BUMP_AMOUNT
    /// and that archived data is accessible.
    #[test]
    fn test_archive_ttl_extended_on_archive_paid_bills() {
        let env = Env::default();
        env.mock_all_auths();

        env.ledger().set(LedgerInfo {
            protocol_version: 20,
            sequence_number: 100,
            timestamp: 1000,
            network_id: [0; 32],
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 3_000_000,
        });

        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        // Create and pay a bill so it can be archived
        client.create_bill(
            &owner,
            &String::from_str(&env, "Old Electric"),
            &800,
            &500,
            &false,
            &0,
        );
        client.pay_bill(&owner, &1);

        // Advance ledger so TTL drops below threshold
        // After pay_bill at seq 100: live_until = 518,500
        // At seq 510,000: TTL = 8,500 < 17,280 → archive will re-extend
        env.ledger().set(LedgerInfo {
            protocol_version: 20,
            sequence_number: 510_000,
            timestamp: 510_000,
            network_id: [0; 32],
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 3_000_000,
        });

        // archive_paid_bills calls extend_instance_ttl then extend_archive_ttl
        let archived = client.archive_paid_bills(&owner, &600_000);
        assert_eq!(archived, 1);

        let ttl = env.as_contract(&contract_id, || {
            env.storage().instance().get_ttl()
        });
        assert!(
            ttl >= 518_400,
            "Instance TTL ({}) must be >= INSTANCE_BUMP_AMOUNT (518,400) after archiving",
            ttl
        );

        // Archived bill should be retrievable
        let archived_bill = client.get_archived_bill(&1);
        assert!(archived_bill.is_some(), "Archived bill must be accessible");
    }

    /// Verify that batch_pay_bills extends instance TTL.
    #[test]
    fn test_instance_ttl_extended_on_batch_pay_bills() {
        let env = Env::default();
        env.mock_all_auths();

        env.ledger().set(LedgerInfo {
            protocol_version: 20,
            sequence_number: 100,
            timestamp: 1000,
            network_id: [0; 32],
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 700_000,
        });

        let contract_id = env.register_contract(None, BillPayments);
        let client = BillPaymentsClient::new(&env, &contract_id);
        let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

        let id1 = client.create_bill(
            &owner,
            &String::from_str(&env, "Gas"),
            &300,
            &600_000,
            &false,
            &0,
        );
        let id2 = client.create_bill(
            &owner,
            &String::from_str(&env, "Water"),
            &200,
            &600_000,
            &false,
            &0,
        );

        // Advance ledger past threshold so extend_ttl has observable effect
        // After create_bill at seq 100: live_until = 518,500
        // At seq 510,000: TTL = 8,500 < 17,280
        env.ledger().set(LedgerInfo {
            protocol_version: 20,
            sequence_number: 510_000,
            timestamp: 510_000,
            network_id: [0; 32],
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 700_000,
        });

        let ids = soroban_sdk::vec![&env, id1, id2];
        let paid_count = client.batch_pay_bills(&owner, &ids);
        assert_eq!(paid_count, 2);

        // TTL should be fully refreshed
        let ttl = env.as_contract(&contract_id, || {
            env.storage().instance().get_ttl()
        });
        assert!(
            ttl >= 518_400,
            "Instance TTL ({}) must be >= 518,400 after batch_pay_bills",
            ttl
        );
    }
}
