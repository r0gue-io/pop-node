use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
	let contracts = PathBuf::from("./integration-tests/contracts/");
	for contract in &get_subcontract_directories(&contracts) {
		let contract_path = contract.join("Cargo.toml");
		Command::new("cargo")
			.args(["contract", "build", "--release", "--manifest-path"])
			.arg(&contract_path)
			.spawn()
			.expect(&format!("Failed to build contract: {:?}", contract_path));
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
