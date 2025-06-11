use std::{
	env, fs,
	path::{Path, PathBuf},
	process,
};

use contract_build::{
	execute, BuildArtifacts, BuildMode, BuildResult, ExecuteArgs, ManifestPath, OutputType,
	Verbosity,
};

fn main() {
	println!("cargo:warning=🔨 build.rs is running");

	let contracts_dir = PathBuf::from("contracts");

	let contract_dirs = match get_subcontract_directories(&contracts_dir) {
		Ok(dirs) => dirs,
		Err(e) => {
			eprintln!("Failed to read contracts directory: {}", e);
			process::exit(1);
		},
	};

	for contract in &contract_dirs {
		if let Err(e) = build_contract(contract) {
			eprintln!("Failed to build contract {}: {}", contract.display(), e);
			process::exit(1);
		}
	}

	println!("cargo:warning=✅ build.rs built {} contracts", contract_dirs.len());
}

// Get subdirectories in the contracts/ folder
fn get_subcontract_directories(contracts_dir: &Path) -> Result<Vec<PathBuf>, String> {
	fs::read_dir(contracts_dir)
		.map_err(|e| format!("Could not read directory '{}': {}", contracts_dir.display(), e))?
		.filter_map(|entry| match entry {
			Ok(entry) if entry.path().is_dir() => Some(Ok(entry.path())),
			Ok(_) => None,
			Err(e) => Some(Err(format!("Error reading directory entry: {}", e))),
		})
		.collect()
}

// Compile the contract using contract-build
fn build_contract(contract_dir: &Path) -> Result<BuildResult, String> {
	println!("cargo:warning=📦 Building contract at {}", contract_dir.display());

	let manifest_path = ManifestPath::new(contract_dir.join("Cargo.toml")).map_err(|e| {
		format!("Could not retrieve manifest path for {}: {}", contract_dir.display(), e)
	})?;

	let args = ExecuteArgs {
		build_artifact: BuildArtifacts::CodeOnly,
		build_mode: BuildMode::Debug,
		manifest_path,
		output_type: OutputType::HumanReadable,
		verbosity: Verbosity::Verbose,
		..Default::default()
	};

	execute(args).map_err(|e| format!("Build failed for {}: {}", contract_dir.display(), e))
}
