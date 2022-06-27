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

    #[ink(event)]
    pub struct VoteCast {
        #[ink(topic)]
        voter: AccountId,
        #[ink(topic)]
        proposal_id: OperationId,
        vote: VoteType,
    }
   

    #[ink(storage)]
    #[derive(Default,SpreadAllocate,TimelockControllerStorage)]
    pub struct Governor {
        #[TimelockControllerStorageField]
        timelock: TimelockControllerData,
        name: Option<String>,
        proposals: Mapping<OperationId, ProposalCore>,
        votes: Mapping<OperationId, ProposalVote>,
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

        fn _emit_vote_cast(
            &self,
            voter: AccountId,
            proposal_id: OperationId,
            vote: VoteType,
        ) {
            self.env()
            .emit_event( VoteCast {
                voter,
                proposal_id,
                vote
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

        fn _cast_vote(
            &mut self, 
            proposal_id: OperationId, 
            vote: VoteType, 
        )  -> Result<(),GovernorError> {
            // does the caller have required voting power?
            let caller = self.env().caller();
            let voting_power = self._get_votes(caller, None);
            if voting_power < 1 {
                return Err(GovernorError::InsufficientVotingPower)
            }       

            //does the proposal exist?
            if !self.proposals.contains(&proposal_id) {
                return Err(GovernorError::ProposalDoesNotExist)
            }

            if self.state(proposal_id) != ProposalState::Active {
                return Err(GovernorError::NotOpenForVoting)
            }

            let mut vote_status = self.votes.get(&proposal_id).unwrap();
            ink_env::debug_println!("_cast_vote: id={:?}", vote_status);

            if vote_status.has_voted.contains(&caller) {
                return Err(GovernorError::HasAlreadyVoted)
            }
            //let proposal = self.proposals.get(&proposal_id).unwrap();

            match vote {
                VoteType::Against => vote_status.votes_against += voting_power,
                VoteType::For     => vote_status.votes_for     += voting_power,
                VoteType::Abstain => vote_status.votes_abstain += voting_power,
            };

            vote_status.has_voted.push(caller);
            self.votes.insert(&proposal_id, &vote_status);

            self._emit_vote_cast(caller,proposal_id,vote);

            Ok(())
            
        }

        fn _execute(&
            self, 
            proposal_id: OperationId
        ) -> Result<(), GovernorError> {
            //does the proposal exist?
            if !self.proposals.contains(&proposal_id) {
                return Err(GovernorError::ProposalDoesNotExist)
            }

            if self.state(proposal_id) != ProposalState::Succeeded {
                return Err(GovernorError::VoteHasNotSucceeded)
            }

            //TODO: finish this....
            Ok(())
        }

        //////////////////////////////
        /// Governor read functions
        #[ink(message)]
        pub fn get_votes(&self, account: AccountId, blocknumber_o: Option<BlockNumber>) -> u32 {
            self._get_votes(account,blocknumber_o)
        }

        #[ink(message)]
        pub fn has_voted(&self, proposal_id: OperationId, account: AccountId) -> bool {
            if !self.votes.contains(&proposal_id) {
                return false;
            }
            let vote_status = self.votes.get(&proposal_id).unwrap();

            vote_status.has_voted.contains(&account)
        }

        #[ink(message)]
        pub fn name(&self) -> Option<String> {
            ink_env::debug_println!("name");
            self.name.clone()
        }

        #[ink(message)]
        pub fn proposal_deadline(&self, proposal_id: OperationId) -> Timestamp {
            assert!(self.proposals.contains(&proposal_id), "Proposal does noet exist");
            
            let proposal = self.proposals.get(&proposal_id).unwrap();

            proposal.vote_end

        }

        #[ink(message)]        
        pub fn proposal_snapshot(&self, proposal_id: OperationId) -> Timestamp {
            assert!(self.proposals.contains(&proposal_id), "Proposal does noet exist");
            
            let proposal = self.proposals.get(&proposal_id).unwrap();

            proposal.vote_start

        }

        #[ink(message)]
        pub fn proposal_votes(&self, proposal_id: OperationId) -> (u32,u32,u32) {
            assert!(self.votes.contains(&proposal_id), "Proposal does noet exist");
            
            let proposal = self.votes.get(&proposal_id).unwrap();

            (proposal.votes_against, proposal.votes_for, proposal.votes_abstain)

        }

        #[ink(message)]
        pub fn state(&self, proposal_id: OperationId) -> ProposalState {
            assert!(self.proposals.contains(&proposal_id), "Proposal does noet exist");
            let proposal = self.proposals.get(&proposal_id).unwrap();

            if proposal.executed {
                return ProposalState::Executed
            }

            if proposal.canceled {
                return ProposalState::Canceled
            }

            if proposal.vote_start > self.env().block_timestamp() {
                return ProposalState::Pending
            }

            if proposal.vote_end > self.env().block_timestamp() {
                return ProposalState::Active
            }

            let vote = self.votes.get(&proposal_id).unwrap();
            if vote.votes_for > vote.votes_against {
                return ProposalState::Succeeded
            }
            
            return ProposalState::Defeated
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
        pub fn cast_vote(
            &mut self, 
            proposal_id: OperationId,
            vote: VoteType,
        ) -> Result<(),GovernorError> {
            self._cast_vote(proposal_id, vote)
        }

        #[ink(message)]
        pub fn execute(&mut self, proposal_id: OperationId) -> Result<(), GovernorError> {
            self._execute(proposal_id)
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
            self.votes.insert(&proposal_id, &ProposalVote::default());

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

        #[cfg(feature = "std")]
        use openbrush::test_utils::{
            change_caller,
            accounts
        };

        #[allow(unused_imports)]
        use crate::governor::{
            ProposalState,
            Governor,
            Transaction,
            OperationId,
            Timestamp,
            AccountId,
            VoteType
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
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            change_caller(accounts.alice);
            let mut governor = Governor::new(Some(String::from("Governor")),0,604800,86400);
            let id = governor.propose(Transaction::default(), "test proposal".to_string()).unwrap();
            assert!(governor.cast_vote(id, VoteType::For).is_ok());

            assert!(governor.has_voted(id, accounts.alice));
            assert!(!governor.has_voted(id, accounts.bob));
        }

        #[ink::test]
        fn name_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            assert_eq!(governor.name(), Some(String::from("Governor")));
        }

        #[ink::test]
        fn proposal_deadline_works() {
            let mut governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            let id = governor.propose(Transaction::default(), "test proposal".to_string()).unwrap();
           
            assert_eq!(governor.proposal_deadline(id),691200)
        }

        #[ink::test]
        fn proposal_snapshot_works() {
            let mut governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            let id = governor.propose(Transaction::default(), "test proposal".to_string()).unwrap();
           
            assert_eq!(governor.proposal_snapshot(id),86400)

        }

        #[ink::test]
        fn proposal_votes_works() {
            let accounts = accounts();
//                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            let mut governor = Governor::new(Some(String::from("Governor")),0,604800,86400);
            let id = governor.propose(Transaction::default(), "test proposal".to_string()).unwrap();
            let vote_result = governor.cast_vote(id, VoteType::For);
            assert!(vote_result.is_ok());

            change_caller(accounts.charlie);
            let (votes_against, votes_for, votes_abstain) = governor.proposal_votes(id);
            assert_eq!(votes_for,1);
            assert_eq!(votes_abstain,0);
            assert_eq!(votes_against,0); 

            change_caller(accounts.alice);
            let vote_result = governor.cast_vote(id, VoteType::Abstain);
            ink_env::debug_println!("vote_result: vote_result={:?}", vote_result);
            // assert!(vote_result.is_ok());
            // let (votes_against, votes_for, votes_abstain) = governor.proposal_votes(id);
            // assert_eq!(votes_for,1);
            // assert_eq!(votes_abstain,1);
            // assert_eq!(votes_against,0); 

            // change_caller(accounts.bob);
            // let vote_result = governor.cast_vote(id, VoteType::Against);
            // assert!(vote_result.is_ok());
            // let (votes_against, votes_for, votes_abstain) = governor.proposal_votes(id);
            // assert_eq!(votes_for,1);
            // assert_eq!(votes_abstain,1);
            // assert_eq!(votes_against,1); 

        }

        //tested in propose_works
        //#[ink::test]
        //fn state_works() {
        //    let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
        //    
        //    
        //    assert_eq!(governor.state(OperationId::default()), ProposalState::Pending);
        //}

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
            let mut governor = Governor::new(Some(String::from("Governor")),0,604800,86400);
            let id = governor.propose(Transaction::default(), "test proposal".to_string()).unwrap();
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 1);    
            
            let vote_result = governor.cast_vote(id, VoteType::For);
            assert!(vote_result.is_ok());
            
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 2);    
            //TODO: add verification of actual event and its content!
            
        }

//        #[ink::test]
//        fn execute_works() {
//            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
//            assert!(governor.execute(0).is_ok())
//        }

        #[ink::test]
        fn propose_works() {
            let mut governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            let id = governor.propose(Transaction::default(), "test proposal".to_string()).unwrap();
            assert_eq!(governor.state(id), ProposalState::Pending);
        
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 1);    
            //TODO: add verification of actual event and its content!
        
        }

        #[ink::test]
        fn hash_proposal_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);

            let id = governor.hash_proposal(Transaction::default(),"test proposal".to_string());
            ink_env::debug_println!("hash_proposal: id={:?}", id.clone());
            assert_ne!(id,OperationId::default())
        }

    }

}