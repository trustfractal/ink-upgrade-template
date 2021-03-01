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

        #[ink(message)]
        pub fn nested(&self) -> Result<AccountId> {
            self.enforce_proxy_call()?;

            Ok(self.proxy)
        }

        fn enforce_proxy_call(&self) -> Result<()> {
            if Self::env().caller() != self.proxy {
                Err(Error::NotCalledFromProxy)
            } else {
                Ok(())
            }
        }
    }

    impl upgradeability::Averager for V1 {
        #[ink(message)]
        fn insert(&mut self, value: i32) {
            self.values.push(value);
        }

        #[ink(message)]
        fn average(&self) -> i32 {
            if self.values.is_empty() { return 0; }

            let mut s = 0;

            for x in &self.values {
                s += x;
            }

            s / self.values.len() as i32
        }
    }
}
