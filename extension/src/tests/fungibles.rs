use crate::mock::{new_test_ext, Assets, AssetsInstance, Test};
use codec::{Decode, Encode};
use sp_runtime::{
	ArithmeticError, DispatchError, ModuleError, TokenError, MAX_MODULE_ERROR_ENCODED_SIZE,
};

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
			let index = <<Test as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::index::<
				Assets,
			>()
			.expect("Every active module has an index in the runtime; qed") as u8;
			let mut error =
				pallet_assets::Error::NotFrozen::<Test, AssetsInstance>.encode();
			error.resize(MAX_MODULE_ERROR_ENCODED_SIZE, 0);
			let error = DispatchError::Module(ModuleError {
				index,
				error: TryInto::try_into(error).expect("should work"),
				message: None,
			});
			let encoded = error.encode();
			let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
			assert_eq!(encoded, vec![3, 2, 18, 0, 0, 0]);
			assert_eq!(
				decoded,
				DispatchError::Module(ModuleError {
					index: 2,
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
