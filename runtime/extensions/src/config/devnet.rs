use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::Contains,
};
pub(crate) use pallet_api::fungibles;
use pop_primitives::AssetId;
use sp_runtime::traits::Dispatchable;

use crate::{CallDispatchHandler, StateReadHandler};

pub trait PopApiExtensionConfig:
	frame_system::Config<RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>>
	+ pallet_assets::Config<Self::AssetInstance, AssetId = AssetId>
	+ fungibles::Config
{
	type AssetInstance;
	type StateReadHandler: StateReadHandler;
	type CallDispatchHandler: CallDispatchHandler;
	type AllowedDispatchCalls: Contains<Self::RuntimeCall>;
}
