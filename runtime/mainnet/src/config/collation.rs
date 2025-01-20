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
    use sp_runtime::traits::Get;

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

    #[test]
    fn aura_uses_sr25519_for_authority_id() {
        assert_eq!(
			TypeId::of::<<Runtime as pallet_aura::Config>::AuthorityId>(),
			TypeId::of::<sp_consensus_aura::sr25519::AuthorityId>(),
		);
    }

    #[test]
    fn aura_max_authorities() {
        assert_eq!(<<Runtime as pallet_aura::Config>::MaxAuthorities as Get<u32>>::get(), 3_600);
    }

    #[test]
    fn aura_disabled_validators_not_used() {
        assert_eq!(
			TypeId::of::<<Runtime as pallet_aura::Config>::DisabledValidators>(),
			TypeId::of::<()>(),
		);
    }

    #[test]
    fn aura_allows_multiple_blocks_per_slot() {
        assert_eq!(
			TypeId::of::<<Runtime as pallet_aura::Config>::AllowMultipleBlocksPerSlot>(),
			TypeId::of::<ConstBool<true>>(),
		);
    }

    #[test]
    fn aura_has_six_second_blocks() {
        assert_eq!(<<Runtime as pallet_aura::Config>::SlotDuration as Get<u64>>::get(), 6_000);
    }
}
