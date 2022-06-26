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
    
    use openbrush::{
        storage::Mapping,
        contracts::timelock_controller::*,
    };
    
    use roosterdao::traits::governor::*;


    #[ink(event)]
    pub struct ProposalCreated {
        #[ink(topic)]
        proposal_id: OperationId,
        #[ink(topic)]
        proposer: AccountId,
        transaction: Transaction,
        description: String,
        vote_start: Timestamp,
        vote_end: Timestamp,
    }
   

    #[ink(storage)]
    #[derive(Default,SpreadAllocate,TimelockControllerStorage)]
    pub struct Governor {
        #[TimelockControllerStorageField]
        timelock: TimelockControllerData,
        name: Option<String>,
        proposals: Mapping<OperationId, ProposalCore>,
        voting_delay: Timestamp,
        voting_period: Timestamp,
    }


    impl Governor {
        #[ink(constructor)]
        pub fn new(
            name: Option<String>,
            voting_delay: Timestamp,
            voting_period: Timestamp,
            execution_delay: Timestamp
        ) -> Self {
            ink_lang::utils::initialize_contract(|instance: &mut Self| {
                instance.name = name;
                instance.voting_delay = voting_delay;
                instance.voting_period = voting_period;

                let caller = instance.env().caller();
                let callee = instance.env().account_id();
                let calee_vec = vec![callee];
                
                // `TimelockController` and `AccessControl` have `_init_with_admin` methods.
                // You need to call it for each trait separately, to initialize everything for these traits.
                AccessControlInternal::_init_with_admin(instance, caller);
                TimelockControllerInternal::_init_with_admin(instance, caller, execution_delay, calee_vec.clone(), calee_vec);
   
            })
        }

        //////////////////////////////
        /// Governor internal
        /// 

        fn _emit_proposal_created(
            &self, 
            proposal_id : OperationId,
            proposer: AccountId,
            transaction: Transaction,
            description: String,
            vote_start: Timestamp,
            vote_end: Timestamp,
        ) {
            self.env()
            .emit_event(ProposalCreated { 
                proposal_id,
                proposer,
                transaction,
                description,
                vote_start,
                vote_end,
             })
        }


        fn _hash_proposal(&self, transaction: Transaction, description_hash: [u8; 32]) -> OperationId {
            TimelockController::hash_operation(self, transaction,None, description_hash)
        }

        fn _hash_description(&self, description: String) -> [u8; 32] {
            self.env().hash_bytes::<Blake2x256>(description.as_bytes())
            
        }

        fn _get_votes(&self, account: AccountId, blocknumber_o: Option<BlockNumber>) -> u32 {
            let blocknumber = match blocknumber_o {
                None => self.env().block_number(),
                Some(bn) => bn
            };
            //TODO: call manager contract
            
            ink_env::debug_println!("getVotes: account={:?} blocknumber={:?}", account, blocknumber);
            1
        }


        //////////////////////////////
        /// Governor read functions
        #[ink(message)]
        pub fn get_votes(&self, account: AccountId, blocknumber_o: Option<BlockNumber>) -> u32 {
            self._get_votes(account,blocknumber_o)
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
        pub fn voting_delay(&self) -> Timestamp {
            ink_env::debug_println!("voting_delay");
            self.voting_delay
        }

        #[ink(message)]
        pub fn voting_period(&self) -> Timestamp {
            ink_env::debug_println!("voting_period");
            self.voting_period
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
        pub fn cast_vote(&self, proposal_id: u128) -> Result<(),GovernorError> {
            ink_env::debug_println!("cast_vote: proposal_id={}", proposal_id);
            Ok(())
        }

        #[ink(message)]
        pub fn execute(&self, proposal_id: u128) -> Result<(), GovernorError> {
            ink_env::debug_println!("execute: proposal_id={}", proposal_id);
            Ok(())
        }

        #[ink(message)]
        pub fn propose(
            &mut self, 
            transaction: Transaction, 
            description: String
        ) -> Result<OperationId, GovernorError>  {

            // does the caller have required voting power?
            let caller = self.env().caller();
            let voting_power = self._get_votes(caller, None);
            if voting_power < 1 {
                return Err(GovernorError::InsufficientVotingPower)
            }            
            
            let description_hash = self._hash_description(description.clone());
            let proposal_id = self._hash_proposal(transaction.clone(), description_hash);

            // is this a new proposal
            if self.proposals.contains(&proposal_id) {
                return Err(GovernorError::ProposalAlreadyExists)
            }


            let proposal = ProposalCore {
                vote_start: self.env().block_timestamp() + self.voting_delay,
                vote_end: self.env().block_timestamp() + self.voting_delay + self.voting_period,
                executed: false,
                canceled: false
            };

            self.proposals.insert(&proposal_id, &proposal);

            self
            ._emit_proposal_created(
                proposal_id,
                self.env().caller(),
                transaction,
                description,
                proposal.vote_start,
                proposal.vote_end
            );

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
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            assert_eq!(governor.name(), Some(String::from("Governor")));
        }


        #[ink::test]
        fn get_votes_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            let block_number = ink_env::block_number::<ink_env::DefaultEnvironment>();
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            assert_eq!(governor.get_votes(accounts.alice, Some(block_number)),1);
        }

        #[ink::test]
        fn has_voted_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            assert_eq!(governor.has_voted(0, accounts.alice), false);
        }

        #[ink::test]
        fn name_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            assert_eq!(governor.name(), Some(String::from("Governor")));
        }

        #[ink::test]
        fn proposal_deadline_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            assert_eq!(governor.proposal_deadline(0),ink_env::block_number::<ink_env::DefaultEnvironment>());
        }

        #[ink::test]
        fn proposal_snapshot_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            assert_eq!(governor.proposal_snapshot(0),ink_env::block_number::<ink_env::DefaultEnvironment>());
        }

        #[ink::test]
        fn proposal_votes_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            assert_eq!(governor.proposal_votes(0), 0);
        }

        #[ink::test]
        fn state_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            assert_eq!(governor.state(0), false);
        }

        #[ink::test]
        fn voting_delay_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            assert_eq!(governor.voting_delay(),86400);
        }

        #[ink::test]
        fn voting_period_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            assert_eq!(governor.voting_period(),604800);
        }

        #[ink::test]
        fn cast_vote_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            assert!(governor.cast_vote(0).is_ok())
        }

        #[ink::test]
        fn execute_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            assert!(governor.execute(0).is_ok())
        }

        // #[ink::test]
        // fn propose_works() {
        //     let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
        //     assert!(governor.propose(0).is_ok())
        // }

        #[ink::test]
        fn hash_proposal_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);

            let id = governor.hash_proposal(Transaction::default(),"test proposal".to_string());
            ink_env::debug_println!("hash_proposal: id={:?}", id.clone());
            assert_ne!(id,OperationId::default())
        }

    }

}