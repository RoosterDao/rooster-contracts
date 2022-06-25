#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]


#[openbrush::contract]
pub mod governor {
    use ink_storage::traits::SpreadAllocate;
    use ink_prelude::vec;
    use ink_prelude::string::{
        String,
    };

    use ink_env::hash::Blake2x256;
    

    // use ink_lang::codegen::{
    //     EmitEvent,
    //     Env,
    // };

    use openbrush::{
        storage::Mapping,
        contracts::timelock_controller::*,
    };
    
    use roosterdao::traits::governor::*;


    #[ink(event)]
    pub struct ProposalCreated {
        proposal_id: OperationId ,
    }
   

    #[ink(storage)]
    #[derive(Default,SpreadAllocate,TimelockControllerStorage)]
    pub struct Governor {
        #[TimelockControllerStorageField]
        timelock: TimelockControllerData,
        name: Option<String>,
        proposals: Mapping<OperationId, ProposalCore>,
    }


    impl Governor {
        #[ink(constructor)]
        pub fn new(name: Option<String>,min_delay: Timestamp) -> Self {
            ink_lang::utils::initialize_contract(|instance: &mut Self| {
                instance.name = name;

                let caller = instance.env().caller();
                
                //TODO: specify nobody as proposer, contract address as executor
                let cal_vec = vec![caller];
                
                // `TimelockController` and `AccessControl` have `_init_with_admin` methods.
                // You need to call it for each trait separately, to initialize everything for these traits.
                AccessControlInternal::_init_with_admin(instance, caller);
                TimelockControllerInternal::_init_with_admin(instance, caller, min_delay, cal_vec.clone(), cal_vec);
   
            })
        }

        //////////////////////////////
        /// Governor internal
        /// 

        fn _emit_proposal_created(&self, proposal_id : OperationId) {
            self.env().emit_event(ProposalCreated { proposal_id })
        }


        fn _hash_proposal(&self, transaction: Transaction, description_hash: [u8; 32]) -> OperationId {
            TimelockController::hash_operation(self, transaction,None, description_hash)
        }

        fn _hash_description(&self, description: String) -> [u8; 32] {
            self.env().hash_bytes::<Blake2x256>(description.as_bytes())
            
        }


        //////////////////////////////
        /// Governor read functions
        #[ink(message)]
        pub fn get_votes(&self, account: AccountId, blocknumber: BlockNumber) -> u32 {
            ink_env::debug_println!("getVotes: account={:?} blocknumber={:?}", account, blocknumber);
            0
        }

        #[ink(message)]
        pub fn has_voted(&self, proposal_id: u128, account: AccountId) -> bool {
            ink_env::debug_println!("has_voted: proposal_id={}, account={:?}",proposal_id, account);
            false
        }

        #[ink(message)]
        pub fn name(&self) -> Option<String> {
            ink_env::debug_println!("name");
            self.name.clone()
        }

        #[ink(message)]
        pub fn proposal_deadline(&self, proposal_id: u128) -> BlockNumber {
            ink_env::debug_println!("proposal_deadline: proposal_id={}", proposal_id);
            ink_env::block_number::<ink_env::DefaultEnvironment>()
        }

        #[ink(message)]        
        pub fn proposal_snapshot(&self, proposal_id: u128) -> BlockNumber {
            ink_env::debug_println!("proposal_snapshot: proposal_id={}", proposal_id);
            ink_env::block_number::<ink_env::DefaultEnvironment>()
        }

        #[ink(message)]
        pub fn proposal_votes(&self, proposal_id: u128) -> u32 {
            ink_env::debug_println!("proposal_votes: proposal_id={}", proposal_id);
            0
        }

        #[ink(message)]
        pub fn state(&self, proposal_id: u128) -> bool {
            ink_env::debug_println!("state: proposal_id={}", proposal_id);
            false
        }

        #[ink(message)]
        pub fn voting_delay(&self) -> u32 {
            ink_env::debug_println!("voting_delay");
            0
        }

        #[ink(message)]
        pub fn voting_period(&self) -> u32 {
            ink_env::debug_println!("voting_period");
            0
        }

        // #[ink(message)]
        // pub fn hash_proposal(&self, transaction: Transaction, description_hash: [u8; 32]) -> OperationId {
        //     self._hash_proposal(transaction, description_hash)
        // }

        #[ink(message)]
        pub fn hash_proposal(&self, transaction: Transaction, description: String) -> OperationId {
            let description_hash = self._hash_description(description);
            self._hash_proposal(transaction, description_hash)
        }

        

        //////////////////////////////
        /// Governor write functions
        #[ink(message)]
        pub fn cast_vote(&self, proposal_id: u128) -> Result<(),MockError> {
            ink_env::debug_println!("cast_vote: proposal_id={}", proposal_id);
            Ok(())
        }

        #[ink(message)]
        pub fn execute(&self, proposal_id: u128) -> Result<(), MockError> {
            ink_env::debug_println!("execute: proposal_id={}", proposal_id);
            Ok(())
        }

        #[ink(message)]
        pub fn propose(&mut self, transaction: Transaction, description: String) -> Result<OperationId, MockError>  {
            let description_hash = self._hash_description(description);
            let proposal_id = self._hash_proposal(transaction, description_hash);

            self.proposals.insert(&proposal_id, &ProposalCore::default());

            self._emit_proposal_created(proposal_id);

            //ink_env::debug_println!("propose: proposal_id={}", proposal_id);
            Ok(proposal_id)
        }
    }

    mod tests {
        use ink_lang as ink;

        #[allow(unused_imports)]
        use crate::governor::{
            Governor,
            Transaction,
            OperationId
        };        

        #[ink::test]
        fn default_works() {
            let governor = Governor::new(Some(String::from("Governor")),0);
            assert_eq!(governor.name(), Some(String::from("Governor")));
        }


        #[ink::test]
        fn get_votes_works() {
            let governor = Governor::new(Some(String::from("Governor")),0);
            let block_number = ink_env::block_number::<ink_env::DefaultEnvironment>();
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            assert_eq!(governor.get_votes(accounts.alice, block_number),0);
        }

        #[ink::test]
        fn has_voted_works() {
            let governor = Governor::new(Some(String::from("Governor")),0);
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            assert_eq!(governor.has_voted(0, accounts.alice), false);
        }

        #[ink::test]
        fn name_works() {
            let governor = Governor::new(Some(String::from("Governor")),0);
            assert_eq!(governor.name(), Some(String::from("Governor")));
        }

        #[ink::test]
        fn proposal_deadline_works() {
            let governor = Governor::new(Some(String::from("Governor")),0);
            assert_eq!(governor.proposal_deadline(0),ink_env::block_number::<ink_env::DefaultEnvironment>());
        }

        #[ink::test]
        fn proposal_snapshot_works() {
            let governor = Governor::new(Some(String::from("Governor")),0);
            assert_eq!(governor.proposal_snapshot(0),ink_env::block_number::<ink_env::DefaultEnvironment>());
        }

        #[ink::test]
        fn proposal_votes_works() {
            let governor = Governor::new(Some(String::from("Governor")),0);
            assert_eq!(governor.proposal_votes(0), 0);
        }

        #[ink::test]
        fn state_works() {
            let governor = Governor::new(Some(String::from("Governor")),0);
            assert_eq!(governor.state(0), false);
        }

        #[ink::test]
        fn voting_delay_works() {
            let governor = Governor::new(Some(String::from("Governor")),0);
            assert_eq!(governor.voting_delay(),0);
        }

        #[ink::test]
        fn voting_period_works() {
            let governor = Governor::new(Some(String::from("Governor")),0);
            assert_eq!(governor.voting_period(),0);
        }

        #[ink::test]
        fn cast_vote_works() {
            let governor = Governor::new(Some(String::from("Governor")),0);
            assert!(governor.cast_vote(0).is_ok())
        }

        #[ink::test]
        fn execute_works() {
            let governor = Governor::new(Some(String::from("Governor")),0);
            assert!(governor.execute(0).is_ok())
        }

        // #[ink::test]
        // fn propose_works() {
        //     let governor = Governor::new(Some(String::from("Governor")),0);
        //     assert!(governor.propose(0).is_ok())
        // }

        #[ink::test]
        fn hash_proposal_works() {
            let governor = Governor::new(Some(String::from("Governor")),0);

            let id = governor.hash_proposal(Transaction::default(),"test proposal".to_string());
            ink_env::debug_println!("hash_proposal: id={:?}", id.clone());
            assert_ne!(id,OperationId::default())
        }

    }

}