#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod proxy {
    use ink_prelude::*;
    use v1::V1;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        UpgradeError,
        UnauthorizedCaller,
    }
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    pub struct Proxy {
        backend: V1,
        owner: AccountId,
    }

    impl Proxy {
        #[ink(constructor)]
        pub fn new(code_hash: Hash) -> Self {
            let backend = V1::new(Self::env().caller())
                .endowment(Self::env().balance() / 2)
                .code_hash(code_hash)
                .salt_bytes(1i32.to_le_bytes())
                .instantiate()
                .expect("failed at instantiating the internal contract");

            Self { backend: backend, owner: Self::env().caller() }
        }

        #[ink(message, payable)]
        pub fn upgrade(&mut self, code_hash: Hash) -> Result<()> {
            use ink_lang::ToAccountId;

            if Self::env().caller() != self.owner {
                return Err(Error::UnauthorizedCaller);
            }

            self.backend = V1::upgrade_from(self.backend.to_account_id(), Self::env().caller())
                .endowment(Self::env().balance() / 2)
                .code_hash(code_hash)
                .salt_bytes(1i32.to_le_bytes())
                .instantiate()
                .map_err(|_e| Error::UpgradeError)?;

            Ok(())
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
