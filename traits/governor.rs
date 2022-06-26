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

    traits::AccountId,
};

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
    // This is kinda ugly, I know... Would rather use a Mapping here, but I have to figure out how...
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
    #[ink(message)]
    fn name(&self) -> String;

    #[ink(message)]
    fn hash_proposal(&self, transaction: Transaction, description_hash: [u8; 32]) -> OperationId;
}
