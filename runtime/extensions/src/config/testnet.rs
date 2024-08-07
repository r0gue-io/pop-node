use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::Contains,
};
use sp_runtime::traits::Dispatchable;

use crate::{CallDispatchHandler, StateReadHandler};
pub trait PopApiExtensionConfig:
	frame_system::Config<RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>>
{
	type StateReadHandler: StateReadHandler;
	type CallDispatchHandler: CallDispatchHandler;
	type AllowedDispatchCalls: Contains<Self::RuntimeCall>;
}
