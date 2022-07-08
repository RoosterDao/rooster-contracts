#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]



use ink_env::{AccountId, Environment};
use ink_lang as ink;
use ink_prelude::vec::Vec;

#[derive(scale::Encode, scale::Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RCErrorCode {
    Failed,
    CollectionNotCreated,
    CollectionAlreadyCreated,
}

#[derive(scale::Encode, scale::Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RCError {
    ErrorCode(RCErrorCode),
}

type CollectionId = u32;
type NftId = u32;
type ResourceId = u32;
#[ink::chain_extension]
pub trait RmrkExt {
    type ErrorCode = RCErrorCode;

    #[ink(extension = 1, returns_result = false)]
    fn read_nft(caller_id: AccountId, collection_id: CollectionId, nft_id: NftId) -> bool;

    #[ink(extension = 2, returns_result = false)]
    fn mint_nft(
        contract_address: AccountId,
        owner: AccountId,
        collection_id: CollectionId,
        metadata: Vec<u8>,
    ) -> Option<NftId>;

    #[ink(extension = 3, returns_result = false)]
    fn create_collection(
        contract_address: AccountId,
        metadata: Vec<u8>,
        symbol: Vec<u8>,
    ) -> Option<CollectionId>;

    #[ink(extension = 4, returns_result = false)]
    fn add_resource(
        contract_address: AccountId,
        collection_id: CollectionId,
        nft_id: NftId,
        metadata: Vec<u8>,
    ) -> Option<ResourceId>;

    #[ink(extension = 5, returns_result = false)]
    fn remove_resource(
        contract_address: AccountId,
        collection_id: CollectionId,
        nft_id: NftId,
        resource_id: ResourceId,
    );
}

impl From<RCErrorCode> for RCError {
    fn from(error_code: RCErrorCode) -> Self {
        Self::ErrorCode(error_code)
    }
}

impl From<scale::Error> for RCError {
    fn from(_: scale::Error) -> Self {
        panic!("encountered unexpected invalid SCALE encoding")
    }
}

impl ink_env::chain_extension::FromStatusCode for RCErrorCode {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::Failed),
            _ => panic!("encountered unknown status code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = RmrkExt;
}

#[openbrush::contract(env = crate::CustomEnvironment)]
pub mod governor {
    use ink_storage::traits::SpreadAllocate;
    use ink_prelude::vec;
    use ink_prelude::vec::Vec;

    use super::{
        RCError, 
        RCErrorCode,
        CollectionId,
        NftId,
    };
    
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

    #[ink(event)]
    pub struct DelegateChanged {
        #[ink(topic)]
        delegator: AccountId,
        from_delegatee: AccountId,
        to_delegatee: AccountId,
    }

    #[ink(event)]
    pub struct DelegateVotesChanged {
        #[ink(topic)]
        delegatee: AccountId,
        votes: u32,
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
        // Temporary implementation 
        delegations: Mapping<BlockNumber, (AccountId, AccountId)>,
        delegation_blocks: Vec<BlockNumber>,
        collection_id: Option<CollectionId>,
    }


    impl Governor {
        #[ink(constructor, payable)]
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


        fn _emit_delegate_changed(
            &self,
            delegator: AccountId,
            to_delegatee: AccountId,
        ) {

            // we need to figure out previous delegatee for delegator
            let mut seen_first = false;
            let mut from_delegatee : AccountId = Default::default();
            for block in self.delegation_blocks.iter().rev() {
                let (cur_delegator, cur_delegatee) = self.delegations.get(&block).unwrap();
                if cur_delegator == delegator {
                    if seen_first {
                        seen_first = false;
                    } else {
                        from_delegatee = cur_delegatee;
                        break;
                    }
                }
            }
            
            self.env()
            .emit_event (
                DelegateChanged {
                    delegator,
                    from_delegatee,
                    to_delegatee,
                })
        }

        fn _emit_delegate_votes_changed(
            &self,
            delegatee: AccountId
        ) {
            let votes = self._get_votes(delegatee, None);

            self.env()
            .emit_event (
                DelegateVotesChanged {
                    delegatee,
                    votes
                })

        }

        fn _hash_proposal(&self, transaction: Transaction, description_hash: [u8; 32]) -> OperationId {
            TimelockController::hash_operation(self, transaction,None, description_hash)
        }

        fn _hash_description(&self, description: String) -> [u8; 32] {
            self.env().hash_bytes::<Blake2x256>(description.as_bytes())
            
        }


        /// Verifies account owns required NFT
        /// 
        /// # Errors
        /// 
        /// 
        ///    `NotOwner` if not 
        fn _has_required_nft(
            &self, 
            caller: AccountId,
            nft_id: NftId
        ) -> Result<(),GovernorError> {
            
            let owner = self.env()
            .extension()
            .read_nft(caller, self.collection_id.unwrap(), nft_id)
            .map_err(|_| false)
            .unwrap();
            

            if !owner {
               return Err(GovernorError::NotOwner)    
            }

            Ok(())
        }

        fn _get_votes(&self, account: AccountId, blocknumber_o: Option<BlockNumber>) -> u32 {
            let block_limit = match blocknumber_o {
                None => self.env().block_number(),
                Some(bn) => bn
            };

            let mut result : u32 = 0;
            let mut already_seen : Vec<AccountId> = Default::default();
            for block in self.delegation_blocks.iter().rev() {
                if block > &block_limit {
                    continue;
                }

                let (cur_delegator, cur_delegatee) = self.delegations.get(&block).unwrap();
                if !already_seen.contains(&cur_delegator) {
                    already_seen.push(cur_delegator);
                    if cur_delegatee == account {
                        result += 1
                    }
                }
            }
            
            ink_env::debug_println!("getVotes: blocknumber={:?} account={:?} result={:?}", block_limit, account, result);
            result
        }

        /// Verifies account has voting power
        ///
        /// # Errors
        ///
        ///     Returns with `InsufficientVotingPower` if voting power is not available
        fn _has_voting_power(&self, caller: AccountId) -> Result<(),GovernorError> {
           let voting_power = self._get_votes(caller, None);
           if voting_power < 1 {
               Err(GovernorError::InsufficientVotingPower)
           }  else {
            Ok(())
           }
        }

       


        fn _cast_vote(
            &mut self, 
            proposal_id: OperationId, 
            vote: VoteType, 
        )  -> Result<(),GovernorError> {
            let caller = self.env().caller();
            self._has_voting_power(caller)?;

            if !self.proposals.contains(&proposal_id) {
                return Err(GovernorError::ProposalDoesNotExist)
            }

            if self.state(proposal_id) != ProposalState::Active {
                return Err(GovernorError::NotOpenForVoting)
            }

            let mut vote_status = self.votes.get(&proposal_id).unwrap();
            
            
            if vote_status.has_voted.contains(&caller) {
                return Err(GovernorError::HasAlreadyVoted)
            }

            let voting_power = self._get_votes(caller, None);
            match vote {
                VoteType::Against => vote_status.votes_against += voting_power,
                VoteType::For     => vote_status.votes_for     += voting_power,
                VoteType::Abstain => vote_status.votes_abstain += voting_power,
            };

            vote_status.has_voted.push(caller);
            self.votes.insert(&proposal_id, &vote_status);
            ink_env::debug_println!("_cast_vote: caller={:?} vote_status={:?}", caller, vote_status);

            self._emit_vote_cast(caller,proposal_id,vote);

            Ok(())
            
        }

        fn _execute(
            &self, 
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

        
        pub fn _create_collection_metadata(
            &mut self,
            metadata: String,
            symbol: String,
        ) -> Result<(), RCError> {
            if self.collection_id != None {
                return Err(RCError::ErrorCode(RCErrorCode::CollectionAlreadyCreated));
            }

            let result = self.env().extension().create_collection(
                self.env().account_id(),
                metadata.into_bytes(),
                symbol.into_bytes(),
            );

            if result.is_err() {
                return Err(RCError::ErrorCode(RCErrorCode::Failed));
            }
            let collection_id = result.unwrap();

            match collection_id {
                Some(cid) => self.collection_id = Some(cid),
                None => return Err(RCError::ErrorCode(RCErrorCode::Failed)),
            }

            Ok(())
        }


        pub fn _create_collection(&mut self) -> Result<(), RCError> {
            let metadata = r#"
            {
                "description": "Rooster DAO",
                "attributes": [],
                "external_url": "project_home_page_url",
                "image": "ipfs://ipfs/QmQUKBhRG7225uJQ5bmUw1UDVxep8fYp4y94hEqCZA5yFN"
              }"#;
            
              let symbol = "ROO";

              self._create_collection_metadata(metadata.into(), symbol.into())
        }


        //////////////////////////////
        /// Governor read functions
        /// 
        
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
            ink_env::debug_println!("voting_delay()");
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

        
        /// ERC721Votes read functions
        #[ink(message)] 
        pub fn get_past_votes(&self, account: AccountId, block: BlockNumber) -> u32 {
            self._get_votes(account, Some(block))
        }

        #[ink(message)]
        pub fn get_votes(&self, account: AccountId) -> u32 {
            self._get_votes(account, None)
        }



        //////////////////////////////
        /// Governor write functions
        /// 
        #[ink(message)]
        pub fn create_collection(&mut self) -> Result<(), RCError> {
            self._create_collection()
        }



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

            let caller = self.env().caller();
            self._has_voting_power(caller)?;


            ink_env::debug_println!("propose(caller={:?}, Transaction={:?}, description={:?})",caller,transaction,description);
            
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

            Ok(proposal_id)
        }

        #[ink(message)]
        pub fn delegate(
            &mut self,
            delegatee: AccountId,
            nft_id: NftId,
        ) -> Result<(),GovernorError> {

            let caller = self.env().caller();
            self._has_required_nft(caller, nft_id)?;

            let current_block = self.env().block_number();
            self.delegations.insert(&current_block, &(caller,delegatee));

            if !self.delegation_blocks.contains(&current_block) {
                self.delegation_blocks.push(current_block);
            }
        
            self._emit_delegate_changed(caller, delegatee);
            self._emit_delegate_votes_changed(delegatee);

           Ok(())
        }

        // #[ink(message,payable)]
        // pub fn become_member(
        //     &mut self,

        // )
        
    }

    mod tests {
        use ink_lang as ink;

        #[allow(unused_imports)]
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
            VoteType,
            GovernorError,
        };        
    

        #[ink::test]
        fn default_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            assert_eq!(governor.name(), Some(String::from("Governor")));
        }


        #[ink::test]
        fn get_votes_works() {
            let accounts = accounts();
            change_caller(accounts.bob);

            let mut governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            //let block_number = ink_env::block_number::<ink_env::DefaultEnvironment>();
            //ink_env::debug_println!("get_votest_works: account={:?}", accounts.bob);

            assert!(governor.delegate(accounts.bob, 2).is_ok());

            assert_eq!(governor.get_votes(accounts.bob),1);
        }

        #[ink::test]
        fn has_voted_works() {
            let accounts = accounts();
            change_caller(accounts.alice);
            let mut governor = Governor::new(Some(String::from("Governor")),0,604800,86400);

            assert!(governor.delegate(accounts.alice, 1).is_ok());

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
            let accounts = accounts();
            change_caller(accounts.alice);

            let mut governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);

            assert!(governor.delegate(accounts.alice, 2).is_ok());
            let id = governor.propose(Transaction::default(), "test proposal".to_string()).unwrap();
           
            assert_eq!(governor.proposal_deadline(id),691200)
        }

        #[ink::test]
        fn proposal_snapshot_works() {
            let accounts = accounts();
            change_caller(accounts.alice);

            let mut governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            assert!(governor.delegate(accounts.alice,1).is_ok());
            let id = governor.propose(Transaction::default(), "test proposal".to_string()).unwrap();
           
            assert_eq!(governor.proposal_snapshot(id),86400)

        }

        #[ink::test]
        fn proposal_votes_works() {
            ink_env::debug_println!("proposal_votes_work:: *** start ***");
            let accounts = accounts();

            change_caller(accounts.bob);
            let mut governor = Governor::new(Some(String::from("Governor")),0,604800,86400);
            assert!(governor.delegate(accounts.bob, 2).is_ok());
            let id = governor.propose(Transaction::default(), "test proposal".to_string()).unwrap();
            let vote_result = governor.cast_vote(id, VoteType::For);
            assert!(vote_result.is_ok());

            let (votes_against, votes_for, votes_abstain) = governor.proposal_votes(id);
            assert_eq!(votes_for,1);
            assert_eq!(votes_abstain,0);
            assert_eq!(votes_against,0); 

            change_caller(accounts.charlie);
            assert!(governor.delegate(accounts.charlie,3 ).is_ok());
            let vote_result = governor.cast_vote(id, VoteType::Abstain);
            assert!(vote_result.is_ok());
            let (votes_against, votes_for, votes_abstain) = governor.proposal_votes(id);
            assert_eq!(votes_for,1);
            assert_eq!(votes_abstain,1);
            assert_eq!(votes_against,0); 

            change_caller(accounts.eve);
            assert!(governor.delegate(accounts.eve,4).is_ok());
            let vote_result = governor.cast_vote(id, VoteType::Against);
            assert!(vote_result.is_ok());
            let (votes_against, votes_for, votes_abstain) = governor.proposal_votes(id);
            assert_eq!(votes_for,1);
            assert_eq!(votes_abstain,1);
            assert_eq!(votes_against,1); 

            change_caller(accounts.alice);
            assert!(governor.delegate(accounts.alice,1).is_ok());
            let vote_result = governor.cast_vote(id, VoteType::For);
            assert!(vote_result.is_ok());
            let (votes_against, votes_for, votes_abstain) = governor.proposal_votes(id);
            assert_eq!(votes_for,2);
            assert_eq!(votes_abstain,1);
            assert_eq!(votes_against,1); 


            ink_env::debug_println!("proposal_votes_work:: *** end ***");

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
            let accounts = accounts();

            change_caller(accounts.bob);

            let mut governor = Governor::new(Some(String::from("Governor")),0,604800,86400);
            assert!(governor.delegate(accounts.bob,2).is_ok());
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 2);    

            let id : OperationId = Default::default();
            assert_eq!(governor.cast_vote(id, VoteType::For),
                       Err(GovernorError::ProposalDoesNotExist));


            let id = governor.propose(Transaction::default(), "test proposal".to_string()).unwrap();
            ink_env::debug_println!("proposal_id = {:?}", id);
            
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 3);    
            
            let vote_result = governor.cast_vote(id, VoteType::For);
            assert!(vote_result.is_ok());
            assert_eq!(governor.cast_vote(id, VoteType::For),Err(GovernorError::HasAlreadyVoted));
            
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 4);    
            //TODO: add verification of actual event and its content!

            change_caller(accounts.eve);
            assert_eq!(governor.cast_vote(id, VoteType::For),Err(GovernorError::InsufficientVotingPower));
            
        }

//        #[ink::test]
//        fn execute_works() {
//            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
//            assert!(governor.execute(0).is_ok())
//        }

        #[ink::test]
        fn propose_works() {
            let accounts = accounts();
            change_caller(accounts.bob);

            let mut governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);
            assert!(governor.delegate(accounts.bob,2).is_ok());
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 2);    


            let id = governor.propose(Transaction::default(), "test proposal".to_string()).unwrap();
            assert_eq!(governor.state(id), ProposalState::Pending);
        
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 3);    
            //TODO: add verification of actual event and its content!

            assert_eq!(governor.propose(Transaction::default(), "test proposal".to_string()), 
                       Err(GovernorError::ProposalAlreadyExists));
        
        }

        #[ink::test]
        fn hash_proposal_works() {
            let governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);

            let id = governor.hash_proposal(Transaction::default(),"test proposal".to_string());
            ink_env::debug_println!("hash_proposal: id={:?}", id.clone());
            assert_ne!(id,OperationId::default())
        }


        #[ink::test]
        fn delegate_works() {
            let accounts = accounts();

            let mut governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);

            change_caller(accounts.bob);
            assert!(governor.delegate(accounts.bob,2).is_ok());
            assert_eq!(governor.get_votes(accounts.bob), 1);
            
            advance_block();
            assert!(governor.delegate(accounts.eve,4).is_ok());
            assert_eq!(governor.get_votes(accounts.bob), 0);
            assert_eq!(governor.get_votes(accounts.eve), 1);

            advance_block();
            change_caller(accounts.eve);
            assert!(governor.delegate(accounts.eve,4).is_ok());
            assert_eq!(governor.get_votes(accounts.bob), 0);
            assert_eq!(governor.get_votes(accounts.eve), 2);

            advance_block();
            change_caller(accounts.alice);
            assert!(governor.delegate(accounts.eve,2).is_ok());
            assert_eq!(governor.get_votes(accounts.bob), 0);
            assert_eq!(governor.get_votes(accounts.eve), 3);
            assert_eq!(governor.get_votes(accounts.alice), 0);
        }

        #[ink::test]
        fn get_past_votes_works() {
            let accounts = accounts();

            let mut governor = Governor::new(Some(String::from("Governor")),86400,604800,86400);

            change_caller(accounts.bob);
            assert!(governor.delegate(accounts.bob,1).is_ok());
            assert_eq!(governor.get_votes(accounts.bob), 1);
            let block_number_1 = ink_env::block_number::<ink_env::DefaultEnvironment>();

            advance_block();
            assert!(governor.delegate(accounts.eve,2).is_ok());
            assert_eq!(governor.get_votes(accounts.bob), 0);
            assert_eq!(governor.get_votes(accounts.eve), 1);
            
            let block_number_2 = ink_env::block_number::<ink_env::DefaultEnvironment>();

            advance_block();
            change_caller(accounts.eve);
            assert!(governor.delegate(accounts.eve,2).is_ok());
            assert_eq!(governor.get_votes(accounts.bob), 0);
            assert_eq!(governor.get_votes(accounts.eve), 2);


            assert_eq!(governor.get_past_votes(accounts.bob, block_number_1), 1);
            assert_eq!(governor.get_past_votes(accounts.eve, block_number_2), 1);
        }

        
 
        #[allow(dead_code)]
        #[cfg(feature = "std")]
        fn advance_block() {
            let _ = ink_env::test::advance_block::<ink_env::DefaultEnvironment>();
        }

    }

}