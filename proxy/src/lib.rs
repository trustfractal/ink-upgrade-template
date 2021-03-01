#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod proxy {
    use ink_prelude::*;
    use upgradeability::Averager;
    use v1::V1;

    #[ink(storage)]
    pub struct Proxy {
        backend: V1,
        values: vec::Vec<i32>,
    }

    impl Proxy {
        #[ink(constructor)]
        pub fn new(address: AccountId) -> Self {
            use ink_env::call::FromAccountId;
            Self { backend: V1::from_account_id(address), values: vec![] }
        }

        #[ink(message)]
        pub fn upgrade(&mut self, address: AccountId) {
            use ink_env::call::FromAccountId;
            self.backend = V1::from_account_id(address);
        }

        #[ink(message)]
        pub fn parent(&self) -> AccountId {
            Self::env().caller()
        }

        #[ink(message)]
        pub fn nested(&self) -> AccountId {
            self.backend.nested().unwrap()
        }
    }

    // good candidate for auto generation
    impl Averager for Proxy {
        #[ink(message)]
        fn insert(&mut self, value: i32) {
            self.backend.insert(value)
        }

        #[ink(message)]
        fn average(&self) -> i32 {
            self.backend.average()
        }
    }
}
