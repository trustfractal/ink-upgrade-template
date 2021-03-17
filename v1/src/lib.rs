#![cfg_attr(not(feature = "std"), no_std)]

pub use self::v1::V1;
use ink_lang as ink;


#[ink::contract]
mod v1 {
    use ink_prelude::*;

    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotCalledFromProxy,
    }
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    pub struct V1 {
        values: vec::Vec<i32>,
        proxy: AccountId,
    }

    impl V1 {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self { values: vec![], proxy: AccountId::default() }
        }

        #[ink(message)]
        pub fn items(&self) -> u32 {
            self.values.len() as u32
        }

        #[ink(message)]
        pub fn nth(&self, idx: u32) -> i32 {
            self.values[idx as usize]
        }

        fn enforce_proxy_call(&self) -> Result<()> {
            if Self::env().caller() != self.proxy {
                Err(Error::NotCalledFromProxy)
            } else {
                Ok(())
            }
        }

        #[ink(message)]
        pub fn insert(&mut self, value: i32) {
            self.values.push(value);
        }

        #[ink(message)]
        pub fn average(&self) -> i32 {
            if self.values.is_empty() {
                return 0;
            }

            self.values.iter().sum::<i32>() / self.values.len() as i32
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::v1::V1;

        #[test]
        fn starts_out_empty() {
            let contract = V1::default();

            assert_eq!(contract.items(), 0);
        }

        #[test]
        fn insert_registers_new_item() {
            let mut contract = V1::default();

            contract.insert(10);

            assert_eq!(contract.items(), 1);
            assert_eq!(contract.nth(0), 10);
        }

        #[test]
        fn average_of_nothing_defaults_to_zero() {
            let contract = V1::default();

            assert_eq!(contract.average(), 0);
        }

        #[test]
        fn average_is_middle_value_when_odd_items() {
            let mut contract = V1::default();

            contract.insert(10);
            contract.insert(50);
            contract.insert(20);

            assert_eq!(contract.average(), (10 + 20 + 50) / 3);
        }
    }
}
