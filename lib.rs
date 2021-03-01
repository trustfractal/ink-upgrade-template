#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::trait_definition]
pub trait Averager {
    #[ink(message)]
    fn insert(&mut self, value: i32);

    #[ink(message)]
    fn average(&self) -> i32;
}
