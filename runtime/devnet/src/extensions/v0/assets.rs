use crate::extensions::{
	AccountId, AssetId,
	AssetsKeys::{self, TotalSupply},
	Balance, Compact, Decode, DispatchError, MultiAddress, Runtime, TrustBackedAssetsInstance,
};

pub(crate) fn construct_assets_key(
	call_index: u8,
	params: Vec<u8>,
) -> Result<AssetsKeys, DispatchError> {
	match call_index {
		0 => {
			let id = <AssetId>::decode(&mut &params[..])
				.map_err(|_| DispatchError::Other("DecodingFailed"))?;
			Ok(TotalSupply(id))
		},
		// other calls
		_ => Err(DispatchError::Other("UnknownFunctionId")),
	}
}

pub(crate) fn construct_assets_call(
	call_index: u8,
	params: Vec<u8>,
) -> Result<pallet_assets::Call<Runtime, TrustBackedAssetsInstance>, DispatchError> {
	match call_index {
		9 => {
			let (id, target, amount) = <(AssetId, AccountId, Balance)>::decode(&mut &params[..])
				.map_err(|_| DispatchError::Other("DecodingFailed"))?;
			Ok(pallet_assets::Call::<Runtime, TrustBackedAssetsInstance>::transfer_keep_alive {
				id: Compact(id),
				target: MultiAddress::Id(target),
				amount,
			})
		},
		// other calls
		_ => Err(DispatchError::Other("UnknownFunctionId")),
	}
}
