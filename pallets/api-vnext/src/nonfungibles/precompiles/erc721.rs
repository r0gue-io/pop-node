use frame_support::sp_runtime::traits::AtLeast32Bit;
use AddressMatcher::Prefix;
use IERC721::*;

use super::*;

sol!("src/nonfungibles/precompiles/interfaces/IERC721.sol");

/// Precompile providing an interface of the ERC-721 standard as defined in the ERC.
pub struct Erc721<const PREFIX: u16, T, I>(PhantomData<(T, I)>);
impl<
		const PREFIX: u16,
		T: frame_system::Config
			+ Config<I, CollectionId: AtLeast32Bit, ItemId: AtLeast32Bit>
			+ pallet_revive::Config,
		I: 'static,
	> Precompile for Erc721<PREFIX, T, I>
{
	type Interface = IERC721Calls;
	type T = T;

	const HAS_CONTRACT_INFO: bool = false;
	const MATCHER: AddressMatcher =
		Prefix(NonZero::new(PREFIX).expect("expected non-zero precompile address"));

	fn call(
		address: &[u8; 20],
		input: &Self::Interface,
		env: &mut impl Ext<T = Self::T>,
	) -> Result<Vec<u8>, Error> {
		use IERC721::{IERC721Calls::*, *};

		let (collection_index, item_index) = InlineCollectionItemExtractor::from_address(address)?;
		match input {
			// IERC721
			balanceOf(_) => {
				unimplemented!()
			},
			ownerOf(_) => {
				unimplemented!()
			},
			safeTransferFrom_0(_) => {
				unimplemented!()
			},
			safeTransferFrom_1(_) => {
				unimplemented!()
			},
			transferFrom(_) => {
				unimplemented!()
			},
			approve(_) => {
				unimplemented!()
			},
			setApprovalForAll(_) => {
				unimplemented!()
			},
			getApproved(_) => {
				unimplemented!()
			},
			isApprovedForAll(_) => {
				unimplemented!()
			},
			// // IERC721Mintable
			// mint(_) => {},
			// // IERC721Burnable
			// burn(_) => {},
			// // IERC721Metadata
			// name(_) => {
			// 	unimplemented!()
			// },
			// symbol(_) => {
			// 	unimplemented!()
			// },
			// tokenURI(_) => {
			// 	unimplemented!()
			// },
			// tokenURI(_) => {
			// 	unimplemented!()
			// },
		}
	}
}

impl<const PREFIX: u16, T: Config<I>, I: 'static> Erc721<PREFIX, T, I> {
	pub fn address(id: u32) -> [u8; 20] {
		prefixed_address(PREFIX, id)
	}
}
