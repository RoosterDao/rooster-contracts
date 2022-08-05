use ink_storage::traits::{
    PackedLayout,
    SpreadLayout,
};
use ink_prelude::string::String;
use ink_prelude::vec::Vec;

use openbrush::{
    contracts::timelock_controller::*,
    traits::Timestamp,
    // storage::{
    //     Mapping,
    //     RawMapping
    // },

    traits::{
        AccountId,
        BlockNumber,
    },
};

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


pub type NftId = u32;
pub type CollectionId = u32;
pub type ResourceId = u32;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ProposalState {
    Pending,
    Active,
    Canceled,
    Defeated,
    Succeeded,
    Queued,
    Expired,
    Executed
}

#[derive(scale::Encode, scale::Decode, Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum GovernorError {
    InsufficientVotingPower,
    ProposalAlreadyExists,
    ProposalDoesNotExist,
    NotOpenForVoting,
    HasAlreadyVoted,
    VoteHasNotSucceeded,
    NotOwner,
    InsufficientAmount,
    AlreadyOwner,
    MintFailed,
    AddResourceFailed,
}

#[derive(scale::Encode, scale::Decode, Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum VoteType {
    Against,
    For,
    Abstain
}

#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;

#[derive(Default, Debug, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
pub struct ProposalVote {
    pub votes_against: u32,
    pub votes_for:     u32,
    pub votes_abstain: u32,
    // Todo: nest mapping 
    pub has_voted: Vec<AccountId>,
}



#[derive(Default, Debug, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
//#[derive(Default, Debug, SpreadLayout,)]
#[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
pub struct ProposalCore {
    pub vote_start: Timestamp,
    pub vote_end:   Timestamp,
    pub executed: bool,
    pub canceled: bool,
}

#[openbrush::trait_definition]
pub trait Governor {
    //read functions
    #[ink(message)]
    fn has_voted(&self, proposal_id: OperationId, account: AccountId) -> bool;

    #[ink(message)]
    fn name(&self) -> String;

    #[ink(message)]
    fn proposal_deadline(&self, proposal_id: OperationId) -> Timestamp;

    #[ink(message)]        
    fn proposal_snapshot(&self, proposal_id: OperationId) -> Timestamp;

    #[ink(message)]
    fn proposal_votes(&self, proposal_id: OperationId) -> (u32,u32,u32);
        
    #[ink(message)]
    fn state(&self, proposal_id: OperationId) -> ProposalState;

    #[ink(message)]
    fn voting_delay(&self) -> Timestamp;

    #[ink(message)]
    fn voting_period(&self) -> Timestamp;

    #[ink(message)]
    fn hash_proposal(&self, transaction: Transaction, description_hash: [u8; 32]) -> OperationId;

    #[ink(message)] 
    fn get_past_votes(&self, account: AccountId, block: BlockNumber) -> u32;

    #[ink(message)]
    fn get_votes(&self, account: AccountId, blocknumber_o: Option<BlockNumber>) -> u32;

    #[ink(message)]
    fn list_owners(&self) -> Vec<(AccountId,NftId,u32)>;


    //write functions
    #[ink(message)]
    fn create_collection(&mut self) -> Result<(), RCError>;

    #[ink(message)]
    fn cast_vote(&mut self, proposal_id: OperationId, vote: VoteType, ) -> Result<(),GovernorError>;

    #[ink(message)]
    fn execute(&mut self, proposal_id: OperationId) -> Result<(), GovernorError>;

    #[ink(message)]
    fn propose(&mut self, transaction: Transaction, description: String) -> Result<OperationId, GovernorError>;

    #[ink(message)]
    fn delegate(&mut self,delegate: AccountId,) -> Result<(),GovernorError>;

    //payable functions
    #[ink(message,payable)]
    fn become_member(&mut self) -> Result<(),GovernorError>;
}
