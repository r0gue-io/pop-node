#!/usr/bin/env bash

# sourced from: https://github.com/paritytech/polkadot-sdk/blob/master/cumulus/scripts/benchmarks-ci.sh

# Script takes 5 arguments as shown below:
# - runtimeType: Type of the runtime: devnet/testnet/mainnet.
# - specName: Name of one of the available genesis presets.
# - artifactsDir: Directory where pop-node is stored.
# - steps: Number of steps to run during benchmarking. Default is 50.
# - repeat: Number of repetitions to run during benchmarking. Default is 20.
#
# Example:
# $ ./becnhmarks-ci.sh mainnet pop ./target/release/ 2 1

# Category is one of: devnet, testnet, mainnet
runtimeType=$1
# One of the available genesis presets.
specName=$2
# Used to source the pop-node binary.
artifactsDir=$3
# Optional number of steps. Default value is 50.
steps=${4:-50}
# Optional number of repetitions. Default value is 20.
repeat=${5:-20}

# Default output directory is the weights folder of the corresponding runtime.
benchmarkOutput=./runtime/$runtimeType/src/weights
# Directory with all benchmarking templates.
benchmarkTemplates="./scripts/templates"

# Load all pallet names in an array.
pallets=($(
  ${artifactsDir}/pop-node benchmark pallet --list --chain="${specName}" |\
    tail -n+2 |\
    cut -d',' -f1 |\
    sort |\
    uniq
))

if [ ${#pallets[@]} -ne 0 ]; then
	echo "[+] Benchmarking ${#pallets[@]} pallets for runtime $runtime"
else
	echo "pallet list not found in benchmarks-ci.sh"
	exit 1
fi

for pallet in ${pallets[@]}
do
	output_dir=""
	extra_args="--template=$benchmarkTemplates/runtime-weight-template.hbs"
	# A little hack for pallet_xcm_benchmarks - we want to force custom implementation for XcmWeightInfo.
	if [[ "$pallet" == "pallet_xcm_benchmarks::generic" ]] || [[ "$pallet" == "pallet_xcm_benchmarks::fungible" ]]; then
		output_dir="xcm/"
		extra_args="--template=$benchmarkTemplates/xcm-bench-template.hbs"
	fi
	$artifactsDir/pop-node benchmark pallet \
		$extra_args \
		--chain=$specName \
		--wasm-execution=compiled \
		--pallet=$pallet  \
		--extrinsic='*' \
		--steps=$steps  \
		--repeat=$repeat \
		--output="${benchmarkOutput}/${output_dir}"
done
