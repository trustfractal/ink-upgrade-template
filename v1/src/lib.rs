#![cfg_attr(not(feature = "std"), no_std)]

pub use self::v1::V1;
use ink_lang as ink;

#[ink::contract]
mod v1 {
    use ink_prelude::*;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotCalledFromProxy,
        UnauthorizedCaller,
    }
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    pub struct V1 {
        values: vec::Vec<i32>,
        proxy: AccountId,
        owner: AccountId,
    }

    impl V1 {
        // new constructs a new empty V1
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
        pub fn upgrade_from(_v1: AccountId, _caller: AccountId) -> Self {
            panic!("not implemented");
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
        pub fn average(&self, _caller: AccountId) -> Result<i32> {
            self.enforce_proxy_call()?;

            Ok(self.average_internal())
        }

        // Internal functions, extracted for ease of testing

        pub fn insert_internal(&mut self, value: i32) {
            self.values.push(value);
        }

        pub fn average_internal(&self) -> i32 {
            if self.values.is_empty() {
                return 0;
            }

            self.values.iter().sum::<i32>() / self.values.len() as i32
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_env::AccountId;
        use ink_lang as ink;

        #[ink::test]
        fn starts_out_empty() {
            let contract = V1::new(AccountId::default());

            assert_eq!(contract.items(), 0);
        }

        #[ink::test]
        fn insert_registers_new_item() {
            let mut contract = V1::new(AccountId::default());

            contract.insert_internal(10);

            assert_eq!(contract.items(), 1);
            assert_eq!(contract.nth(0), 10);
        }

        #[ink::test]
        fn average_of_nothing_defaults_to_zero() {
            let contract = V1::new(AccountId::default());

            assert_eq!(contract.average_internal(), 0);
        }

        #[ink::test]
        fn average_is_middle_value_when_odd_items() {
            let mut contract = V1::new(AccountId::default());

            contract.insert_internal(10);
            contract.insert_internal(50);
            contract.insert_internal(20);

            assert_eq!(contract.average_internal(), (10 + 20 + 50) / 3);
        }

        #[ink::test]
        fn insert_only_called_from_proxy() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

            let proxy = accounts.alice;
            let owner = accounts.bob;
            let other = accounts.eve;

            let mut contract = V1::new(owner);

            call_as(proxy, || {
                contract.insert(10, owner).expect("can't call from proxy");
            });
            call_as(owner, || {
                contract
                    .insert(10, owner)
                    .expect_err("can't call from owner");
            });
            call_as(other, || {
                contract
                    .insert(10, owner)
                    .expect_err("can't call from random");
            });
        }

        #[ink::test]
        fn average_only_called_from_proxy() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

            let proxy = accounts.alice;
            let owner = accounts.bob;
            let other = accounts.eve;

            let contract = V1::new(owner);

            call_as(proxy, || {
                contract.average(owner).expect("can't call from proxy");
            });
            call_as(owner, || {
                contract.average(owner).expect_err("can't call from owner");
            });
            call_as(other, || {
                contract.average(owner).expect_err("can't call from random");
            });
        }

        fn call_as<F>(account: AccountId, mut body: F)
        where
            F: FnMut() -> (),
        {
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                account,
                ink_env::account_id::<ink_env::DefaultEnvironment>().unwrap(),
                1000000,
                1000000,
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])),
            );

            body()
        }
    }
}
