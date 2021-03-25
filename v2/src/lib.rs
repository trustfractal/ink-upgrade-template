#![cfg_attr(not(feature = "std"), no_std)]
#![feature(is_sorted)]

pub use self::v2::V2;
use ink_lang as ink;

#[ink::contract]
mod v2 {
    use ink_prelude::*;
    use v1::V1;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotCalledFromProxy,
        UnauthorizedCaller,
    }
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    pub struct V2 {
        values: vec::Vec<i32>,
        proxy: AccountId,
        owner: AccountId,
    }

    impl V2 {
        // new constructs a new empty V2
        #[ink(constructor)]
        pub fn new(caller: AccountId) -> Self {
            Self {
                values: vec![],
                owner: caller,
                proxy: Self::env().caller(),
            }
        }

        // upgrade_from constructs a new V2 contract based on the data of a given V1 contract
        #[ink(constructor)]
        pub fn upgrade_from(v1: AccountId, _caller: AccountId) -> Self {
            use ink_env::call::FromAccountId;
            let previous = V1::from_account_id(v1);

            let mut new = Self {
                values: vec![],
                owner: previous.owner(),
                proxy: Self::env().caller(),
            };

            for i in 0..previous.items() {
                new.insert_internal(previous.nth(i));
            }

            new
        }

        // Helper authorization functions

        fn enforce_proxy_call(&self) -> Result<()> {
            if Self::env().caller() != self.proxy {
                Err(Error::NotCalledFromProxy)
            } else {
                Ok(())
            }
        }

        fn enforce_owner_call(&self, caller: AccountId) -> Result<()> {
            if caller != self.owner {
                Err(Error::UnauthorizedCaller)
            } else {
                Ok(())
            }
        }

        // Functions to expose the internal state so that future versions can build their own data

        #[ink(message)]
        pub fn items(&self) -> u32 {
            self.values.len() as u32
        }

        #[ink(message)]
        pub fn nth(&self, idx: u32) -> i32 {
            self.values[idx as usize]
        }

        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        // Contract messages

        #[ink(message)]
        pub fn insert(&mut self, value: i32, caller: AccountId) -> Result<()> {
            self.enforce_proxy_call()?;
            self.enforce_owner_call(caller)?;

            Ok(self.insert_internal(value))
        }

        #[ink(message)]
        pub fn average(&self) -> Result<i32> {
            self.enforce_proxy_call()?;

            Ok(self.average_internal())
        }

        // Internal functions, extracted for ease of testing

        pub fn insert_internal(&mut self, value: i32) {
            let idx = self.values.binary_search(&value).unwrap_or_else(|x| x);

            self.values.insert(idx, value);
        }

        pub fn average_internal(&self) -> i32 {
            let n = self.values.len();
            if n == 0 {
                0
            } else if n % 2 == 1 {
                self.values[n / 2]
            } else {
                (self.values[n / 2 - 1] + self.values[n / 2]) / 2
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_env::AccountId;
        use ink_lang as ink;

        #[ink::test]
        fn starts_out_empty() {
            let contract = V2::new(AccountId::default());

            assert_eq!(contract.items(), 0);
        }

        #[ink::test]
        fn insert_registers_new_item() {
            let mut contract = V2::new(AccountId::default());

            contract.insert_internal(10);

            assert_eq!(contract.items(), 1);
            assert_eq!(contract.nth(0), 10);
        }

        #[ink::test]
        fn insert_keeps_things_sorted() {
            let mut contract = V2::new(AccountId::default());

            contract.insert_internal(4);
            contract.insert_internal(10);
            contract.insert_internal(0);

            assert!(contract.values.iter().is_sorted());
        }

        #[ink::test]
        fn average_of_nothing_defaults_to_zero() {
            let contract = V2::new(AccountId::default());

            assert_eq!(contract.average_internal(), 0);
        }

        #[ink::test]
        fn average_is_middle_value_when_odd_items() {
            let mut contract = V2::new(AccountId::default());

            contract.insert_internal(10);
            contract.insert_internal(50);
            contract.insert_internal(20);

            assert_eq!(contract.average_internal(), 20);
        }

        #[ink::test]
        fn average_is_mean_of_middle_values_when_even_items() {
            let mut contract = V2::new(AccountId::default());

            contract.insert_internal(50);
            contract.insert_internal(20);
            contract.insert_internal(10);
            contract.insert_internal(100);

            assert_eq!(contract.average_internal(), (20 + 50) / 2);
        }
    }
}
