use ink_storage::traits::{
    PackedLayout,
    SpreadLayout,
};
use ink_prelude::string::String;


use openbrush::contracts::timelock_controller::*;
use openbrush::traits::Timestamp;

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
pub enum MockError {
    NotOwner,
    NotApproved,
    NotAllowed,
}


#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;

#[derive(Default, Debug, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
pub struct ProposalCore {
    vote_start: Timestamp,
    vote_end:   Timestamp,
    executed: bool,
    canceled: bool,
}

#[openbrush::trait_definition]
pub trait Governor {
    #[ink(message)]
    fn name(&self) -> String;

    #[ink(message)]
    fn hash_proposal(&self, transaction: Transaction, description_hash: [u8; 32]) -> OperationId;
}
