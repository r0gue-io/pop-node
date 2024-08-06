use crate::{
	config::assets::TrustBackedAssetsInstance,
	fungibles::{self},
	Runtime,
};

impl fungibles::Config for Runtime {
	type AssetsInstance = TrustBackedAssetsInstance;
	type WeightInfo = fungibles::weights::SubstrateWeight<Runtime>;
}
