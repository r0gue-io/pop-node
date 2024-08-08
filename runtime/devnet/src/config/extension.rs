use crate::{
	config::api::AllowedApiCalls,
	fungibles::{self},
	Runtime,
};
use codec::Decode;
use frame_support::{ensure, traits::Contains};
use pop_runtime_extension::{
	constants::{DECODING_FAILED_ERROR, UNKNOWN_CALL_ERROR},
	StateReadHandler,
};

use sp_runtime::DispatchError;
use sp_std::vec::Vec;

use super::api::RuntimeRead;

/// Wrapper to enable versioning of runtime state reads.
#[derive(Decode, Debug)]
enum VersionedStateRead {
	/// Version zero of state reads.
	#[codec(index = 0)]
	V0(RuntimeRead),
}

pub struct ContractExecutionContext;

impl StateReadHandler for ContractExecutionContext {
	fn handle_params<T>(params: Vec<u8>) -> Result<Vec<u8>, DispatchError>
	where
		T: pop_runtime_extension::Config,
	{
		let read =
			<VersionedStateRead>::decode(&mut &params[..]).map_err(|_| DECODING_FAILED_ERROR)?;
		match read {
			VersionedStateRead::V0(read) => {
				ensure!(AllowedApiCalls::contains(&read), UNKNOWN_CALL_ERROR);
				match read {
					RuntimeRead::Fungibles(key) => fungibles::Pallet::read_state(key),
				}
			},
		};
		Ok(vec![])
	}
}

impl pop_runtime_extension::Config for Runtime {
	type StateReadHandler = ContractExecutionContext;
	type AllowedDispatchCalls = AllowedApiCalls;
}

#[cfg(test)]
mod tests {

	use super::*;

	use crate::{config::assets::TrustBackedAssetsInstance, Assets, Runtime, System};
	use codec::Encode;
	use sp_runtime::{
		ArithmeticError, BuildStorage, DispatchError, ModuleError, TokenError,
		MAX_MODULE_ERROR_ENCODED_SIZE,
	};

	fn new_test_ext() -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	#[test]
	fn encoding_decoding_dispatch_error() {
		new_test_ext().execute_with(|| {
			let error = DispatchError::Module(ModuleError {
				index: 255,
				error: [2, 0, 0, 0],
				message: Some("error message"),
			});
			let encoded = error.encode();
			let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
			assert_eq!(encoded, vec![3, 255, 2, 0, 0, 0]);
			assert_eq!(
				decoded,
				// `message` is skipped for encoding.
				DispatchError::Module(ModuleError {
					index: 255,
					error: [2, 0, 0, 0],
					message: None
				})
			);

			// Example pallet assets Error into ModuleError.
			let index = <<Runtime as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::index::<
				Assets,
			>()
			.expect("Every active module has an index in the runtime; qed") as u8;
			let mut error =
				pallet_assets::Error::NotFrozen::<Runtime, TrustBackedAssetsInstance>.encode();
			error.resize(MAX_MODULE_ERROR_ENCODED_SIZE, 0);
			let error = DispatchError::Module(ModuleError {
				index,
				error: TryInto::try_into(error).expect("should work"),
				message: None,
			});
			let encoded = error.encode();
			let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
			assert_eq!(encoded, vec![3, 52, 18, 0, 0, 0]);
			assert_eq!(
				decoded,
				DispatchError::Module(ModuleError {
					index: 52,
					error: [18, 0, 0, 0],
					message: None
				})
			);

			// Example DispatchError::Token
			let error = DispatchError::Token(TokenError::UnknownAsset);
			let encoded = error.encode();
			let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
			assert_eq!(encoded, vec![7, 4]);
			assert_eq!(decoded, error);

			// Example DispatchError::Arithmetic
			let error = DispatchError::Arithmetic(ArithmeticError::Overflow);
			let encoded = error.encode();
			let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
			assert_eq!(encoded, vec![8, 1]);
			assert_eq!(decoded, error);
		});
	}
}
