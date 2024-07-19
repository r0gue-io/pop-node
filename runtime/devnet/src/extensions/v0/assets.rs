use crate::extensions::{
	AccountId as AccountId32, AssetId,
	AssetsKeys::{self, *},
	Balance, Compact, Decode, DispatchError, MultiAddress, Runtime, TrustBackedAssetsInstance,
};
use pop_primitives::AccountId;
use sp_std::vec::Vec;

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
		1 => {
			let (id, owner) = <(AssetId, AccountId)>::decode(&mut &params[..])
				.map_err(|_| DispatchError::Other("DecodingFailed"))?;
			Ok(BalanceOf(id, owner))
		},
		2 => {
			let (id, owner, spender) = <(AssetId, AccountId, AccountId)>::decode(&mut &params[..])
				.map_err(|_| DispatchError::Other("DecodingFailed"))?;
			Ok(Allowance(id, owner, spender))
		},
		3 => {
			let id = <AssetId>::decode(&mut &params[..])
				.map_err(|_| DispatchError::Other("DecodingFailed"))?;
			Ok(TokenName(id))
		},
		4 => {
			let id = <AssetId>::decode(&mut &params[..])
				.map_err(|_| DispatchError::Other("DecodingFailed"))?;
			Ok(TokenSymbol(id))
		},
		5 => {
			let id = <AssetId>::decode(&mut &params[..])
				.map_err(|_| DispatchError::Other("DecodingFailed"))?;
			Ok(TokenDecimals(id))
		},
		// other calls
		_ => Err(DispatchError::Other("UnknownFunctionId")),
	}
}

pub(crate) fn construct_assets_call(
	call_index: u8,
	params: Vec<u8>,
	// ) -> Result<pallet_assets::Call<Runtime, TrustBackedAssetsInstance>, DispatchError> {
) -> Result<pallet_fungibles::Call<Runtime>, DispatchError> {
	match call_index {
		9 => {
			let (id, target, amount) = <(AssetId, AccountId32, Balance)>::decode(&mut &params[..])
				.map_err(|_| DispatchError::Other("DecodingFailed"))?;
			// Ok(pallet_assets::Call::<Runtime, TrustBackedAssetsInstance>::transfer_keep_alive {
			// 	id: Compact(id),
			// 	target: MultiAddress::Id(target),
			// 	amount,
			// })
			Ok(pallet_fungibles::Call::<Runtime>::transfer {
				id: Compact(id),
				target: MultiAddress::Id(target),
				amount,
			})
		},
		// 22 => {
		// 	let (id, delegate, amount) =
		// 		<(AssetId, AccountId32, Balance)>::decode(&mut &params[..])
		// 			.map_err(|_| DispatchError::Other("DecodingFailed"))?;
		// 	Ok(pallet_assets::Call::<Runtime, TrustBackedAssetsInstance>::approve_transfer {
		// 		id: Compact(id),
		// 		delegate: MultiAddress::Id(delegate),
		// 		amount,
		// 	})
		// },
		// other calls
		_ => Err(DispatchError::Other("UnknownFunctionId")),
	}
}
