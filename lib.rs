#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::trait_definition]
pub trait Averager {
    #[ink(message)]
    fn get(&self) -> bool;

    #[ink(message)]
    fn flip(&mut self);
}
