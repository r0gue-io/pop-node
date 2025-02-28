frame_benchmarking::define_benchmarks!(
	// Ordered as per runtime
	// System
	[frame_system, SystemBench::<Runtime>]
	[frame_system_extensions, SystemExtensionsBench::<Runtime>]
	[cumulus_pallet_parachain_system, ParachainSystem]
	[pallet_timestamp, Timestamp]
	// Monetary
	[pallet_balances, Balances]
	[pallet_transaction_payment, TransactionPayment]
	[pallet_treasury, Treasury]
	// Governance
	[pallet_sudo, Sudo]
	[pallet_collective, Council]
	[pallet_motion, Motion]
	// Collation support
	[pallet_collator_selection, CollatorSelection]
	[pallet_session, SessionBench::<Runtime>]
	// Scheduler
	[pallet_scheduler, Scheduler]
	// Preimage
	[pallet_preimage, Preimage]
	// XCM
	[cumulus_pallet_xcmp_queue, XcmpQueue]
	// TODO: intro xcm benchmarks
	//[pallet_xcm, PolkadotXcm::<Runtime>]
	[pallet_message_queue, MessageQueue]
	// Contracts
	[pallet_revive, Revive]
	// Proxy
	[pallet_proxy, Proxy]
	// Multisig
	[pallet_multisig, Multisig]
	// Utility
	[pallet_utility, Utility]
	// Assets
	[pallet_nfts, Nfts]
	[pallet_assets, Assets]
);
