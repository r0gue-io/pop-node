use pop_runtime_common::SLOT_DURATION;

use crate::{
    Aura, AuraId, CollatorSelection, ConstBool, ConstU32,
    ConstU64, Runtime,
};

impl pallet_authorship::Config for Runtime {
    type EventHandler = (CollatorSelection,);
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
}

#[docify::export(aura_config)]
impl pallet_aura::Config for Runtime {
    type AllowMultipleBlocksPerSlot = ConstBool<true>;
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<3600>;
    type SlotDuration = ConstU64<SLOT_DURATION>;
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use super::*;

    #[test]
    fn authorship_finds_block_author_via_index_from_digests_within_block_header() {
        assert_eq!(
			TypeId::of::<<Runtime as pallet_authorship::Config>::FindAuthor>(),
			TypeId::of::<pallet_session::FindAccountFromAuthorIndex<Runtime, Aura>>(),
		);
    }

    #[test]
    fn authorship_notes_block_author_via_collator_selection() {
        assert_eq!(
			TypeId::of::<<Runtime as pallet_authorship::Config>::EventHandler>(),
			TypeId::of::<(CollatorSelection,)>(),
		);
    }
}
