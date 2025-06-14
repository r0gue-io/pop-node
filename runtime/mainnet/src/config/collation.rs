use pop_runtime_common::{HOURS, SLOT_DURATION};

use crate::{
	parameter_types, weights, AccountId, Aura, AuraId, Balances, CollatorSelection, ConstBool,
	ConstU32, ConstU64, EnsureRoot, PalletId, Runtime, RuntimeEvent, Session, SessionKeys,
};

impl pallet_authorship::Config for Runtime {
	type EventHandler = (CollatorSelection,);
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
}

parameter_types! {
	/// 6 Hours in number of blocks.
	pub const Period: u32 = 6 * HOURS;
}

#[docify::export(aura_config)]
impl pallet_aura::Config for Runtime {
	type AllowMultipleBlocksPerSlot = ConstBool<true>;
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	// With 6 seconds per block and 6h sessions, aura can rotate 3_600 authorities.
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
	// Should be a multiple of session or things will get inconsistent.
	type KickThreshold = Period;
	#[cfg(feature = "runtime-benchmarks")]
	// If configured to `0`, benchmarks underflows.
	type MaxCandidates = ConstU32<10>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MaxCandidates = ConstU32<0>;
	type MaxInvulnerables = ConstU32<20>;
	type MinEligibleCollators = ConstU32<3>;
	type PotId = PotId;
	type RuntimeEvent = RuntimeEvent;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type ValidatorId = AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = weights::pallet_collator_selection::WeightInfo<Runtime>;
}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl pallet_session::Config for Runtime {
	type DisablingStrategy = ();
	type Keys = SessionKeys;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type RuntimeEvent = RuntimeEvent;
	// We delegate to Aura's session handler.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type SessionManager = CollatorSelection;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type ValidatorId = AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type WeightInfo = weights::pallet_session::WeightInfo<Runtime>;
}

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use sp_core::{crypto::Ss58Codec, ByteArray};
	use sp_runtime::traits::{AccountIdConversion, Get};

	use super::*;

	mod authorship {
		use super::*;
		#[test]
		fn notes_block_author_via_collator_selection() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_authorship::Config>::EventHandler>(),
				TypeId::of::<(CollatorSelection,)>(),
			);
		}

		#[test]
		fn finds_block_author_via_index_from_digests_within_block_header() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_authorship::Config>::FindAuthor>(),
				TypeId::of::<pallet_session::FindAccountFromAuthorIndex<Runtime, Aura>>(),
			);
		}
	}

	mod aura {
		use super::*;

		#[test]
		fn allows_multiple_blocks_per_slot() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_aura::Config>::AllowMultipleBlocksPerSlot>(),
				TypeId::of::<ConstBool<true>>(),
			);
		}

		#[test]
		fn uses_sr25519_for_authority_id() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_aura::Config>::AuthorityId>(),
				TypeId::of::<sp_consensus_aura::sr25519::AuthorityId>(),
			);
		}

		#[test]
		fn disabled_validators_not_used() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_aura::Config>::DisabledValidators>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn max_authorities_is_3600() {
			assert_eq!(
				<<Runtime as pallet_aura::Config>::MaxAuthorities as Get<u32>>::get(),
				3_600
			);
		}

		#[test]
		fn has_six_second_blocks() {
			assert_eq!(<<Runtime as pallet_aura::Config>::SlotDuration as Get<u64>>::get(), 6_000);
		}
	}

	mod collator_selection {
		use super::*;

		#[test]
		fn uses_native_asset() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_collator_selection::Config>::Currency>(),
				TypeId::of::<Balances>(),
			);
		}

		#[test]
		fn kick_threshold_matches_period() {
			assert_eq!(
				<<Runtime as pallet_collator_selection::Config>::KickThreshold as Get<u32>>::get(),
				Period::get(),
			);
		}

		#[test]
		fn candidates_disabled() {
			#[cfg(feature = "runtime-benchmarks")]
			assert_eq!(
				<<Runtime as pallet_collator_selection::Config>::MaxCandidates as Get<u32>>::get(),
				10
			);
			// Disabled to start until sufficient distribution/value to allow candidates to provide
			// candidacy bond
			#[cfg(not(feature = "runtime-benchmarks"))]
			assert_eq!(
				<<Runtime as pallet_collator_selection::Config>::MaxCandidates as Get<u32>>::get(),
				0
			);
		}

		#[test]
		fn allows_max_twenty_invulnerables() {
			// Additional invulnerables can be added after genesis via `UpdateOrigin`
			assert_eq!(
				<<Runtime as pallet_collator_selection::Config>::MaxInvulnerables as Get<u32>>::get(
				),
				20
			);
		}

		#[test]
		fn requires_at_least_three_collators() {
			assert_eq!(
				<<Runtime as pallet_collator_selection::Config>::MinEligibleCollators as Get<
					u32,
				>>::get(),
				3
			);
		}

		#[test]
		fn distributes_block_rewards_via_pot() {
			// Context: block author receives rewards from 'pot', less ED. A keyless account
			// 'pot' is generated from the `PotId` value configured for the pallet.

			assert_eq!(
				TypeId::of::<<Runtime as pallet_collator_selection::Config>::PotId>(),
				TypeId::of::<PotId>(),
			);

			assert_eq!(CollatorSelection::account_id(), PotId::get().into_account_truncating());
		}

		#[test]
		fn pot_account_is_valid() {
			// "PotStake" module id to address via  https://www.shawntabrizi.com/substrate-js-utilities/
			let expected =
				AccountId::from_ss58check("5EYCAe5cKPAoFh2HnQQvpKqRYZGqBpaA87u4Zzw89qPE58is")
					.unwrap();
			assert_eq!(CollatorSelection::account_id(), expected);
		}

		#[test]
		fn update_origin_limited_to_root() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_collator_selection::Config>::UpdateOrigin>(),
				TypeId::of::<EnsureRoot<<Runtime as frame_system::Config>::AccountId>>(),
			);
		}

		#[test]
		fn identifies_collators_using_account_id() {
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
		fn ensures_session_keys_registered() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_collator_selection::Config>::ValidatorRegistration>(
				),
				TypeId::of::<Session>(),
			);
		}

		#[test]
		fn does_not_use_default_weights() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_collator_selection::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}
	}

	mod session {
		use super::*;

		#[test]
		fn ensures_no_disabling_strategy() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_session::Config>::DisablingStrategy>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn keys_provided_by_aura() {
			// Session keys implementation uses aura-defined authority identifier type
			SessionKeys {
				aura: <Runtime as pallet_aura::Config>::AuthorityId::from_slice(&[0u8; 32])
					.unwrap(),
			};
			assert_eq!(
				TypeId::of::<<Runtime as pallet_session::Config>::Keys>(),
				TypeId::of::<SessionKeys>(),
			);
		}

		#[test]
		fn length_is_predefined_period_of_blocks() {
			assert_eq!(Period::get(), 6 * HOURS);
			assert_eq!(Period::get(), (60 / 6) * 60 * 6); // 6s blocks per minute * minutes in an hour * hours
			let periodic_sessions =
				TypeId::of::<pallet_session::PeriodicSessions<Period, Offset>>();

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
		fn handled_by_aura() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_session::Config>::SessionHandler>(),
				TypeId::of::<(Aura,)>(),
			);
		}

		#[test]
		fn collators_managed_by_collator_selection() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_session::Config>::SessionManager>(),
				TypeId::of::<CollatorSelection>(),
			);
		}

		#[test]
		fn identifies_collators_using_account_id() {
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
		fn does_not_use_default_weights() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_session::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}
	}
}
