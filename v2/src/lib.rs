#![cfg_attr(not(feature = "std"), no_std)]

pub use self::v2::V2;
use ink_lang as ink;


#[ink::contract]
mod v2 {
    use ink_prelude::*;
    use v1::V1;

    #[ink(storage)]
    pub struct V2 {
        sorted_values: vec::Vec<i32>,
    }

    impl V2 {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self { sorted_values: vec![] }
        }

        pub fn from_v1(address: AccountId) -> Self {
            use ink_env::call::FromAccountId;
            let previous = V1::from_account_id(address);

            let mut new = Self {
                sorted_values: vec![],
            };

            for i in 0..previous.items() {
                new.sorted_values.push(previous.nth(i));
            }

            new
        }
    }

    impl upgradeability::Averager for V2 {
        #[ink(message)]
        fn insert(&mut self, value: i32) {
            let idx = self
                .sorted_values
                .binary_search(&value)
                .unwrap_or_else(|x| x);

            self.sorted_values.insert(idx, value);
        }

        #[ink(message)]
        fn average(&self) -> i32 {
            if self.sorted_values.is_empty() { return 0; }

            self.sorted_values[self.sorted_values.len() / 2]
        }
    }
}
