use super::RuntimeCall;
use crate::*;
use xcm::{VersionedLocation, VersionedXcm, VersionedAssets, Version as XcmVersion, v4::Location, v3::WeightLimit};


#[derive(scale::Encode)]
pub(crate) enum XcmCalls {
    #[codec(index = 0)]
    Send {
        dest: VersionedLocation,
        message: VersionedXcm<()>,
    },
    #[codec(index = 1)]
    TeleportAssets {
        dest: VersionedLocation,
        beneficiary: VersionedLocation,
        assets: VersionedAssets,
        fee_asset_item: u32,
    },
    #[codec(index = 2)]
    ReserveTransferAssets {
        dest: VersionedLocation,
        beneficiary: VersionedLocation,
        assets: VersionedAssets,
        fee_asset_item: u32,
    },
    // #[codec(index = 3)]
    // Execute {
    //     message: Box<RuntimeCall>,
	// 	max_weight: Weight,
    // },
    #[codec(index = 4)]
    ForceXcmVersion {
        location: Location,
        version: XcmVersion
    },
    #[codec(index = 5)]
    ForceDefaultXcmVersion {
        maybe_xcm_version: Option<XcmVersion>,
    },
    #[codec(index = 6)]
    ForceSubscribeVersionNotify {
        location: VersionedLocation,
    },
    #[codec(index = 7)]
    ForceUnsubscribeVersionNotify {
        location: VersionedLocation,
    },
    #[codec(index = 8)]
    LimitedReserveTransferAssets {
        dest: VersionedLocation,
        beneficiary: VersionedLocation,
        assets: VersionedAssets,
        fee_asset_item: u32,
        weight_limit: WeightLimit,
    },
    #[codec(index = 9)]
    LimitedTeleportAssets {
        dest: VersionedLocation,
        beneficiary: VersionedLocation,
        assets: VersionedAssets,
        fee_asset_item: u32,
        weight_limit: WeightLimit,
    },
    #[codec(index = 10)]
    ForceSuspension {
        suspended: bool,
    },
    #[codec(index = 11)]
    TransferAssets {
        dest: VersionedLocation,
        beneficiary: VersionedLocation,
        assets: VersionedAssets,
        fee_asset_item: u32,
        weight_limit: WeightLimit,
    },
    #[codec(index = 12)]
    ClaimAssets {
        assets: VersionedAssets,
        beneficiary: VersionedLocation,
    },
}