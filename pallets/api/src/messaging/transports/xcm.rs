pub(crate) use xcm::latest::{Location, QueryId, Response};

use crate::messaging::{pallet::Call, BlockNumberOf, Config};
use xcm_builder::QueryControllerWeightInfo;

pub trait NotifyQueryHandler<T: Config> {
	type WeightInfo: QueryControllerWeightInfo;
	/// Attempt to create a new query ID and register it as a query that is yet to respond, and
	///  which will call a dispatchable when a response happens.
	fn new_notify_query(
		responder: impl Into<Location>,
		notify: Call<T>,
		timeout: BlockNumberOf<T>,
		match_querier: impl Into<Location>,
	) -> u64;
}
