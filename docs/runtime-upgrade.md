# Runtime Upgrade

This section describes how to perform a runtime upgrade.

## Requirements

- Sudo keys
- New WASM file of the upgraded runtime (
  e.g. `./target/release/wbuild/pop-runtime-testnet/pop_runtime_testnet.compact.compressed.wasm`)
- [Polkadot/Substrate Portal](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frpc2.paseo.popnetwork.xyz#/explorer)

### Generating new WASM file

> Note: the relevant deterministic build artifacts are automatically generated and attached to each runtime release, so
> the below manual steps are not strictly necessary.

- Ensure that the required changes have been made and merged into the `main` branch. Note the last commit hash.
- Navigate to the **Actions** page of the source code repository on GitHub -
  e.g. https://github.com/r0gue-io/pop-node/actions.
- Select **Build Deterministic Runtimes** on the left and then select **Run workflow** on the right and select **Run
  workflow** on the `main` branch.
- Once the workflow has completed successfully, select it and then select the `testnet-runtime-commithash` artifact to
  download it (provided the commit hash matches that of the `main` branch noted above).
- Open the `testnet-srtool-digest.json` file and make note of the `runtimes/compressed/blake2_256` value as we will use
  this to check against the hash of the corresponding `testnet_runtime.compact.compressed.wasm` file in the same
  directory.

## Polkadot/Substrate Portal

Go to the Polkadot/Substrate portal and connect to Pop Network Paseo.
In addition, in another tab connect
to [Paseo](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fpaseo-rpc.dwellir.com#/explorer) (for following the upgrade
process).

## Extrinsics

### Step 1: Authorize the Upgrade

Whilst connected to Pop Network Paseo, at the top of the screen select `Developer` and then `Sudo`.

Submit the following change:

- Select `system` & `authorizeUpgrade(codeHash)`.
- `CodeHash: H256 (Hash)`: on the right, enable `hash a file`. Select the
  new `pop_runtime_testnet.compact.compressed.wasm`
  file from the file system (generated above).
- Verify that the resulting code hash matches that of the value noted during the runtime wasm generation above.
- On the bottom right find and select `Submit Sudo`.
- On the bottom right of the screen that popped up, find and select `Sign and Submit`.
- Enter the password with the Polkadot JS Extension and `Sign the Transaction`.

At the top of the screen select `Network` and then `Explorer`. On the right of the screen find `recent events`.
The event `system.UpgradeAuthorized` should appear within 6 seconds.

### Step 2: Apply the Upgrade

Go to `Developer` and select `Extrinsics`.

Submit the following change:

- Select `system` & `applyAuthorizedUpgrade(code)`
- `Code: Bytes`: on the right, enable `file upload`. Select the same `pop_runtime_testnet.compact.compressed.wasm` file
  from
  the file system.
- On the bottom right find and select `Submit Unsigned`.

At the top of the screen select `Network` and then `Explorer`.
Go to `recent events`, the event `system.CodeUpdated` should appear within 6 seconds.

You can monitor the upgrade process by viewing the parachain status
at [Parachains on Paseo](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fpaseo-rpc.dwellir.com#/parachains).

### Runtime Upgraded

Go to `Network` and click `Explorer`. Go to `recent events`, after a few minutes the
event `parachainSystem.ValidationFunctionApplied` should appear.

The runtime is now successfully upgraded :)

Note: you may need to refresh the page if certain chain state queries do not seem to update after the upgrade.
