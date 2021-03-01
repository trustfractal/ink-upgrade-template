#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod proxy {
    use upgradeability::Averager;
    use v1::V1;

    #[ink(storage)]
    pub struct Proxy {
        backend: V1,
    }

    impl Proxy {
        #[ink(constructor)]
        pub fn new(address: AccountId) -> Self {
            use ink_env::call::FromAccountId;
            Self { backend: V1::from_account_id(address) }
        }

        #[ink(message)]
        pub fn upgrade(&mut self, address: AccountId) {
            use ink_env::call::FromAccountId;
            self.backend = V1::from_account_id(address);

            // migrate data somehow
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
