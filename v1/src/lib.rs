#![cfg_attr(not(feature = "std"), no_std)]

pub use self::v1::V1;
use ink_lang as ink;

#[ink::contract]
mod v1 {
    #[ink(storage)]
    pub struct V1 {
        values: Vec<i32>,
    }

    impl V1 {
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }
    }

    impl upgradeability::Flipper for V1 {
        #[ink(message)]
        fn flip(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        fn get(&self) -> bool {
            self.value
        }
    }
}
