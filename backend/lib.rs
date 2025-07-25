use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};
use std::cell::RefCell;

type Memory = VirtualMemory<DefaultMemoryImpl>;

// To store global state in a Rust canister, we use the `thread_local!` macro.
thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // We store the greeting in a `Cell` in stable memory such that it gets persisted over canister upgrades.
    static GREETING: RefCell<ic_stable_structures::Cell<String, Memory>> = RefCell::new(
        ic_stable_structures::Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), "Hello, ".to_string()
        ).unwrap()
    );
}

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod spark_up {
    use super::*;

    #[ink(storage)]
    pub struct SparkUp {
        ideas: ink_storage::Mapping<u32, Idea>,
        contributions: ink_storage::Mapping<(u32, AccountId), Balance>,
        total_ideas: u32,
    }

    #[derive(scale::Encode, scale::Decode, Clone, Debug, Default)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Idea {
        title: String,
        description: String,
        owner: AccountId,
        fund_goal: Balance,
        deadline: Timestamp,
        amount_collected: Balance,
        completed: bool,
    }

    impl SparkUp {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                ideas: Default::default(),
                contributions: Default::default(),
                total_ideas: 0,
            }
        }

        /// Creates a new crowdfunding idea
        #[ink(message)]
        pub fn create_idea(
            &mut self,
            title: String,
            description: String,
            fund_goal: Balance,
            deadline: Timestamp,
        ) {
            self.total_ideas += 1;
            let caller = self.env().caller();
            let idea = Idea {
                title,
                description,
                owner: caller,
                fund_goal,
                deadline,
                amount_collected: 0,
                completed: false,
            };
            self.ideas.insert(self.total_ideas, &idea);
        }

        /// Contribute to an idea
        #[ink(message, payable)]
        pub fn fund_idea(&mut self, id: u32) {
            let idea = self.ideas.get(id).expect("Idea not found");
            assert!(!idea.completed, "Idea already completed");
            assert!(idea.deadline > self.env().block_timestamp(), "Deadline passed");

            let amount = self.env().transferred_value();
            assert!(amount > 0, "Amount must be greater than 0");

            let mut updated_idea = idea.clone();
            updated_idea.amount_collected += amount;

            let sender = self.env().caller();
            let key = (id, sender);
            let prev_contribution = self.contributions.get(key).unwrap_or(0);
            self.contributions.insert(key, &(prev_contribution + amount));
            self.ideas.insert(id, &updated_idea);
        }

        /// Returns the details of an idea
        #[ink(message)]
        pub fn get_idea(&self, id: u32) -> Option<Idea> {
            self.ideas.get(id)
        }
    }
}


// Export the interface for the smart contract.
ic_cdk::export_candid!();
