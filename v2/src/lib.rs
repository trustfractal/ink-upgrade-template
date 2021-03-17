#![cfg_attr(not(feature = "std"), no_std)]
#![feature(is_sorted)]

pub use self::v2::V2;
use ink_lang as ink;


#[ink::contract]
mod v2 {
    use ink_prelude::*;
    use v1::V1;

    #[ink(storage)]
    pub struct V2 {
        values: vec::Vec<i32>,
        proxy: AccountId,
    }

    impl V2 {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self { values: vec![], proxy: AccountId::default() }
        }

        pub fn from_v1(address: AccountId) -> Self {
            use ink_env::call::FromAccountId;

            let previous = V1::from_account_id(address);

            let mut new = Self::default();
            for i in 0..previous.items() {
                new.insert(previous.nth(i));
            }

            new
        }

        #[ink(message)]
        pub fn items(&self) -> u32 {
            self.values.len() as u32
        }

        #[ink(message)]
        pub fn nth(&self, idx: u32) -> i32 {
            self.values[idx as usize]
        }

        #[ink(message)]
        pub fn insert(&mut self, value: i32) {
            let idx = self
                .values
                .binary_search(&value)
                .unwrap_or_else(|x| x);

            self.values.insert(idx, value);
        }

        #[ink(message)]
        pub fn average(&self) -> i32 {
            if self.values.is_empty() {
                return 0;
            }

            let n = self.values.len();

            if n % 2 == 1 {
                self.values[n / 2]
            } else {
                (self.values[n / 2 - 1] + self.values[n / 2]) / 2
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::v2::V2;

        #[test]
        fn starts_out_empty() {
            let contract = V2::default();

            assert_eq!(contract.items(), 0);
        }

        #[test]
        fn insert_registers_new_item() {
            let mut contract = V2::default();

            contract.insert(10);

            assert_eq!(contract.items(), 1);
            assert_eq!(contract.nth(0), 10);
        }

        #[test]
        fn insert_keeps_things_sorted() {
            let mut contract = V2::default();

            contract.insert(4);
            contract.insert(10);
            contract.insert(0);

            assert!(contract.values.iter().is_sorted());
        }


        #[test]
        fn average_of_nothing_defaults_to_zero() {
            let contract = V2::default();

            assert_eq!(contract.average(), 0);
        }

        #[test]
        fn average_is_middle_value_when_odd_items() {
            let mut contract = V2::default();

            contract.insert(10);
            contract.insert(50);
            contract.insert(20);

            assert_eq!(contract.average(), 20);
        }

        #[test]
        fn average_is_mean_of_middle_values_when_even_items() {
            let mut contract = V2::default();

            contract.insert(50);
            contract.insert(20);
            contract.insert(10);
            contract.insert(100);

            assert_eq!(contract.average(), (20 + 50) / 2);
        }
    }
}
