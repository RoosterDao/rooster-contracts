use ink_storage::traits::{
    PackedLayout,
    SpreadLayout,
};

use ink_prelude::string::String;

#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;

#[derive(Default, Debug, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
pub struct ProposalCore {
    proposal: String,
    executed: bool,
    canceled: bool,
}

#[openbrush::trait_definition]
pub trait Governor {
    #[ink(message)]
    fn name(&self) -> String;
}
