#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod proxy {
    use ink_prelude::*;
    use v1::V1;

    #[ink(storage)]
    pub struct Proxy {
        backend: V1,
    }

    impl Proxy {
        #[ink(constructor)]
        pub fn new(code_hash: Hash) -> Self {
            let backend = V1::new(Self::env().account_id(), Self::env().caller())
                .endowment(Self::env().balance() / 2)
                .code_hash(code_hash)
                .salt_bytes(1i32.to_le_bytes())
                .instantiate()
                .expect("failed at instantiating the internal contract");

            Self { backend: backend }
        }

        #[ink(message)]
        pub fn upgrade(&mut self, code_hash: Hash) {
            use ink_lang::ToAccountId;

            self.backend = V1::upgrade_from(
                    Self::env().account_id(),
                    Self::env().caller(),
                    self.backend.to_account_id(),
                )
                .endowment(Self::env().balance() / 2)
                .code_hash(code_hash)
                .salt_bytes(1i32.to_le_bytes())
                .instantiate()
                .expect("failed at instantiating the internal contract");
        }

        #[ink(message)]
        pub fn insert(&mut self, value: i32) {
            self.backend.insert(value, Self::env().caller()).unwrap()
        }

        #[ink(message)]
        pub fn average(&self) -> i32 {
            self.backend.average(Self::env().caller()).unwrap()
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::proxy::Proxy;

        #[test]
        fn starts_out_empty() {
            let contract = Proxy::default();

            assert_eq!(contract.items(), 0);
        }
    }
}
