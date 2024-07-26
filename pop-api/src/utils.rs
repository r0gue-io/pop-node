use ink::env::chain_extension::ChainExtensionMethod;

pub(crate) fn build_extension_method(
	version: u8,
	function: u8,
	module: u8,
	dispatchable: u8,
) -> ChainExtensionMethod<(), (), (), false> {
	ChainExtensionMethod::build(u32::from_le_bytes([version, function, module, dispatchable]))
}
