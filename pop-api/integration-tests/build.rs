use contract_build::{execute, BuildMode, ExecuteArgs, ManifestPath, Verbosity};
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
	let contracts = PathBuf::from("./contracts/");
	for contract in &get_subcontract_directories(&contracts) {
		let manifest_path = ManifestPath::new(contract.join("Cargo.toml"))
			.expect(&format!("Failed to retrieve contract: {:?}", contract));
		let args = ExecuteArgs {
			build_mode: BuildMode::Release,
			manifest_path,
			verbosity: Verbosity::Quiet,
			..Default::default()
		};
		execute(args).expect(&format!("Failed to build contract: {:?}", contract));
	}
}

// Function to retrieve all subdirectories in a given directory.
fn get_subcontract_directories(contracts_dir: &Path) -> Vec<PathBuf> {
	let mut directories = Vec::new();
	if let Ok(entries) = fs::read_dir(contracts_dir) {
		for entry in entries {
			if let Ok(entry) = entry {
				let path = entry.path();
				if path.is_dir() {
					directories.push(path);
				}
			}
		}
	} else {
		panic!("Failed to read contracts directory: {:?}", contracts_dir);
	}
	directories
}
