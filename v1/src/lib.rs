#![cfg_attr(not(feature = "std"), no_std)]

pub use self::v1::V1;
use ink_lang as ink;


#[ink::contract]
mod v1 {
    use ink_prelude::*;

    #[ink(storage)]
    pub struct V1 {
        values: vec::Vec<i32>,
    }

    impl V1 {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self { values: vec![] }
        }
    }

    impl upgradeability::Averager for V1 {
        #[ink(message)]
        fn insert(&mut self, value: i32) {
            self.values.push(value);
        }

        #[ink(message)]
        fn average(&self) -> i32 {
            let mut s = 0;

            for x in &self.values {
                s += x;
            }

            s / self.values.len() as i32
        }
    }
}
