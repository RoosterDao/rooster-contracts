#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]


#[openbrush::contract]
pub mod governor {
    use ink_storage::traits::SpreadAllocate;
    use ink_prelude::string::{
        String,
        ToString,
    };

    
    use openbrush::storage::Mapping;
    
    use roosterdao::traits::governor::*;


    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Governor {
        proposals: Mapping<AccountId, ProposalCore>,
    }

    impl Governor {
        #[ink(constructor)]
        pub fn new() -> Self {
            // This call is required in order to correctly initialize the
            // `Mapping`s of our contract.
            ink_lang::utils::initialize_contract(|_| {})
        }

        #[ink(message)]
        pub fn name(&self) -> String {
            //ink_env::debug_println!("name");
            "name".to_string()
        }



    }

}