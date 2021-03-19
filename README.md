## Upgradeability in ink contracts

This document describes a way to add upgradeability to your smart contracts.

There are several limitations to this approach, described below. You should
read and understand them before applying this approach to your contracts.

The mechanism used here is based on the [Proxy pattern][proxy-pattern] used in
ethereum. It works by having two contracts: the proxy contract, which is the
one that other accounts will interact with, and the internal contract, which
contains most of the smart contract logic. You upgrade your contract by
switching the internal contract referenced by the proxy, making the upgrade
process transparent for the end users of your contract.


## Limitations

You cannot add or change the signature of methods in your contract with this
upgrade mechanism. This is defined in the proxy contract and cannot be changed.
This includes the constructor of the internal contract. It must follow a
specific signature, since it is called by the proxy.

This approach doesn't handle balance transfers. This means that any currency
sent to the proxy won't be forwarded to the internal contract, and any upgrade
examples won't migrate any currency.

When building your internal contracts, you'll need to create methods to expose
its internal state so you can migrate it to a new one. If you don't, you'll
need to reconstruct the state in the new contract indirectly.

There's nothing stopping you from upgrading to a contract that does not match
the signatures required for the proxy to work. Even though we're using rust to
implement the contracts, once they're uploaded to the chain, they're referenced
as code hashes or account ids, so there's no way to enforce a type check when
upgrading.

The `caller`, from the internal contract's point of view, will be the proxy
contract. To remedy this, since most contracts have some authorization
mechanism in place, the proxy passes its caller as an extra argument to the
internal contract.

On each upgrade, the code of the internal contract potentially changes. This
means that the execution cost of each method may change. If other accounts that
interact with your contract, (particularly automated ones) have set execution
limits, they may stop working.

Access to the internal contract should be restricted to the proxy address. We
don't want random folks calling the internal contract after it's no longer the
active one. Everything should go through the proxy, except if we want to allow
for self destructing or augmenting the contract trait somehow. This doesn't
feel very scalable, though.

## How to

In this section we'll go through the process of adding upgradeability to a
sample contract. The contract will have two methods: one to insert an `i32`
value, and another one to calculate the average of the inserted values. Only
the owner of the contract should be able to insert new values, with `average`
being callable by anyone. In the first version, we'll be using the [arithmetic
mean](https://en.wikipedia.org/wiki/Arithmetic_mean). Afterwards, we'll upgrade
it to use the [median](https://en.wikipedia.org/wiki/Median), with a slight
change in storage to make it more efficient.

The first step is to implement the contract as if there's no upgradeability:

~~~~rust
#![cfg_attr(not(feature = "std"), no_std)]

pub use self::v1::V1;
use ink_lang as ink;

#[ink::contract]
mod v1 {
    use ink_prelude::*;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        UnauthorizedCaller,
    }
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    pub struct V1 {
        values: vec::Vec<i32>,
        owner: AccountId,
    }

    impl V1 {
        // new constructs a new empty V1
        #[ink(constructor)]
        pub fn new(owner: AccountId) -> Self {
            Self { values: vec![], owner: owner }
        }

        fn enforce_owner_call(&self, caller: AccountId) -> Result<()> {
            if caller != self.owner {
                Err(Error::UnauthorizedCaller)
            } else {
                Ok(())
            }
        }

        #[ink(message)]
        pub fn insert(&mut self, value: i32) -> Result<()> {
            self.enforce_owner_call(Self::env().caller())?;

            Ok(self.insert_internal(value))
        }

        #[ink(message)]
        pub fn average(&self) -> Result<i32> {
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
        // your tests go here
    }
}
