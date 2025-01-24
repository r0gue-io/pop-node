use pop_runtime_common::{HOURS, SLOT_DURATION};

use crate::{
	parameter_types, AccountId, Aura, AuraId, Balances, CollatorSelection, ConstBool, ConstU32,
	ConstU64, EnsureRoot, PalletId, Runtime, RuntimeEvent, Session, SessionKeys,
};

impl pallet_authorship::Config for Runtime {
	type EventHandler = (CollatorSelection,);
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
}

#[docify::export(aura_config)]
impl pallet_aura::Config for Runtime {
	type AllowMultipleBlocksPerSlot = ConstBool<true>;
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	// Number of blocks we can produce per session.
	type MaxAuthorities = Period;
	type SlotDuration = ConstU64<SLOT_DURATION>;
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const Offset: u32 = 0;
}

/// We allow root to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EnsureRoot<AccountId>;

impl pallet_collator_selection::Config for Runtime {
	type Currency = Balances;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type MaxCandidates = ConstU32<0>;
	type MaxInvulnerables = ConstU32<20>;
	type MinEligibleCollators = ConstU32<3>;
	type PotId = PotId;
	type RuntimeEvent = RuntimeEvent;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = pallet_collator_selection::weights::SubstrateWeight<Runtime>;
}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl pallet_session::Config for Runtime {
	type Keys = SessionKeys;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type RuntimeEvent = RuntimeEvent;
	// Essentially just Aura, but let's be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type SessionManager = CollatorSelection;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use sp_core::{crypto::Ss58Codec, ByteArray};
	use sp_runtime::traits::{AccountIdConversion, Get};

	use super::*;

	#[test]
	fn authorship_notes_block_author_via_collator_selection() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_authorship::Config>::EventHandler>(),
			TypeId::of::<(CollatorSelection,)>(),
		);
	}

	#[test]
	fn authorship_finds_block_author_via_index_from_digests_within_block_header() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_authorship::Config>::FindAuthor>(),
			TypeId::of::<pallet_session::FindAccountFromAuthorIndex<Runtime, Aura>>(),
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
	fn aura_uses_sr25519_for_authority_id() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_aura::Config>::AuthorityId>(),
			TypeId::of::<sp_consensus_aura::sr25519::AuthorityId>(),
		);
	}

	#[test]
	fn aura_disabled_validators_not_used() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_aura::Config>::DisabledValidators>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn aura_max_authorities() {
		assert_eq!(<<Runtime as pallet_aura::Config>::MaxAuthorities as Get<u32>>::get(), 3_600);
	}

	#[test]
	fn aura_has_six_second_blocks() {
		assert_eq!(<<Runtime as pallet_aura::Config>::SlotDuration as Get<u64>>::get(), 6_000);
	}

	#[test]
	fn collator_selection_uses_native_asset() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_collator_selection::Config>::Currency>(),
			TypeId::of::<Balances>(),
		);
	}

	#[test]
	fn collator_selection_kick_threshold_matches_period() {
		assert_eq!(
			<<Runtime as pallet_collator_selection::Config>::KickThreshold as Get<u32>>::get(),
			Period::get(),
		);
	}

	#[test]
	fn collator_selection_candidates_disabled() {
		// Disabled to start until sufficient distribution/value to allow candidates to provide
		// candidacy bond
		assert_eq!(
			<<Runtime as pallet_collator_selection::Config>::MaxCandidates as Get<u32>>::get(),
			0
		);
	}

	#[test]
	fn collator_selection_allows_max_twenty_invulnerables() {
		// Additional invulnerables can be added after genesis via `UpdateOrigin`
		assert_eq!(
			<<Runtime as pallet_collator_selection::Config>::MaxInvulnerables as Get<u32>>::get(),
			20
		);
	}

	#[test]
	fn collator_selection_requires_at_least_three_collators() {
		assert_eq!(
			<<Runtime as pallet_collator_selection::Config>::MinEligibleCollators as Get<u32>>::get(
			),
			3
		);
	}

	#[test]
	fn collator_selection_distributes_block_rewards_via_pot() {
		// Context: block author receives rewards from 'pot', less ED. A keyless account
		// 'pot' is generated from the `PotId` value configured for the pallet.
		assert_eq!(
			TypeId::of::<<Runtime as pallet_collator_selection::Config>::PotId>(),
			TypeId::of::<PotId>(),
		);
		assert_eq!(CollatorSelection::account_id(), PotId::get().into_account_truncating());
	}

	#[test]
	fn collator_selection_pot_account_is_valid() {
		// "PotStake" module id to address via  https://www.shawntabrizi.com/substrate-js-utilities/
		let expected =
			AccountId::from_ss58check("5EYCAe5cKPAoFh2HnQQvpKqRYZGqBpaA87u4Zzw89qPE58is").unwrap();
		assert_eq!(CollatorSelection::account_id(), expected);
	}

	#[test]
	fn collator_selection_update_origin_limited_to_root() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_collator_selection::Config>::UpdateOrigin>(),
			TypeId::of::<EnsureRoot<<Runtime as frame_system::Config>::AccountId>>(),
		);
	}

	#[test]
	fn collator_selection_identifies_collators_using_account_id() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_collator_selection::Config>::ValidatorId>(),
			TypeId::of::<<Runtime as frame_system::Config>::AccountId>(),
		);
		assert_eq!(
			TypeId::of::<<Runtime as pallet_collator_selection::Config>::ValidatorIdOf>(),
			TypeId::of::<pallet_collator_selection::IdentityCollator>(),
		);
	}

	#[test]
	fn collator_selection_ensures_session_keys_registered() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_collator_selection::Config>::ValidatorRegistration>(),
			TypeId::of::<Session>(),
		);
	}

	#[test]
	fn collator_selection_does_not_use_default_weights() {
		assert_ne!(
			TypeId::of::<<Runtime as pallet_collator_selection::Config>::WeightInfo>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn session_keys_provided_by_aura() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_session::Config>::Keys>(),
			TypeId::of::<SessionKeys>(),
		);
		// Session keys implementation uses aura-defined authority identifier type
		SessionKeys {
			aura: <Runtime as pallet_aura::Config>::AuthorityId::from_slice(&[0u8; 32]).unwrap(),
		};
	}

	#[test]
	fn session_length_is_predefined_period_of_blocks() {
		assert_eq!(Period::get(), 6 * HOURS);
		assert_eq!(Period::get(), (60 / 6) * 60 * 6); // 6s blocks per minute * minutes in an hour * hours
		let periodic_sessions = TypeId::of::<pallet_session::PeriodicSessions<Period, Offset>>();

		assert_eq!(
			TypeId::of::<<Runtime as pallet_session::Config>::NextSessionRotation>(),
			periodic_sessions,
		);
		assert_eq!(
			TypeId::of::<<Runtime as pallet_session::Config>::ShouldEndSession>(),
			periodic_sessions,
		);
	}

	#[test]
	fn session_handled_by_aura() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_session::Config>::SessionHandler>(),
			TypeId::of::<(Aura,)>(),
		);
	}

	#[test]
	fn session_collators_managed_by_collator_selection() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_session::Config>::SessionManager>(),
			TypeId::of::<CollatorSelection>(),
		);
	}

	#[test]
	fn session_identifies_collators_using_account_id() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_session::Config>::ValidatorId>(),
			TypeId::of::<<Runtime as frame_system::Config>::AccountId>(),
		);
		assert_eq!(
			TypeId::of::<<Runtime as pallet_session::Config>::ValidatorIdOf>(),
			TypeId::of::<pallet_collator_selection::IdentityCollator>(),
		);
	}

	#[test]
	fn session_does_not_use_default_weights() {
		assert_ne!(
			TypeId::of::<<Runtime as pallet_session::Config>::WeightInfo>(),
			TypeId::of::<()>(),
		);
	}
}
