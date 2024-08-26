---
name: New Release
about: Create a tracking issue for a new release.
title: '<node/runtime>-v<X.Y.Z>'
assignees: ''
---

## Release Readiness Tracking

> _Node Release_

- [ ] All changes have passed peer-review. < link PR >
- [ ] `pop-node` crate version has been updated.
- [ ] The new binary is able to sync the live network and peers with other active nodes, logs are healthy.
- [ ] One collator runs with the new binary, syncs the network and peers with other nodes, produces blocks and logs are healthy.

> _Runtime Release_

- [ ] All changes have passed peer-review. < link PR >
- [ ] Update runtime crate version. Note that `pop-runtime-devnet` is usually not updated.
- [ ] Runtime spec version is updated.
- [ ] If needed, new benchmarks have been run. A diff between the new weights and the current ones has been reviewed.
    - [`substrate-weight-compare`](https://github.com/ggwpez/substrate-weight-compare) can be used for this purpose.
- [ ] Execution of [`try-runtime`](https://github.com/paritytech/try-runtime-cli) doesn't point out any missing migrations or other items requiring action.



## Testing Tracking

- [ ] Local upgrade test runs as expected.

1. Build the latest release.
2. Launch a network using `pop up parachiain -f networks/testnet.toml -v`.
    - `pop-node` version can be verified via: `rpc calls -> system -> version()`
    - Runtime version can be verified via: `rpc calls -> state -> getRuntimeVersion()`


> _Node release_

3. Switch to the new release branch and rebuild.
4. Kill the running `pop-node` process
> For instance, on Mac one can look in Activity Monitor, find the `pop-node` process and force quit. The network will still be running, but without the collator so Pop Network might not be producing blocks.
6. Run the new `pop-node` binary using the command prompted in step `2`, with the same specs that are prompted at step `2`.
7. Verify that the new node is producing blocks.
8. Verify running versions as needed.

> _Runtime release_

_The new runtime might need a certian `pop-node` version to be deployed first, if that is the case make sure you follow the above steps for a Node Release_

3. Do a runtime upgrade using the new runtime release -- can be found in `./target/release/wbuild/pop-runtime-testnet/pop_runtime_testnet.compact.compressed.wasm`.
4. Verify the runtime upgrade was successful:
    - The runtime version should have changed.
    - The corresponding migrations should have run.
    - Pop Network is still producing blocks.


- [ ] Successful execution of `try-runtime` .
```
cargo build --release --features=try-runtime -p <runtime-crate>
try-runtime --runtime ./target/release/wbuild/pop-runtime-testnet/pop_runtime_testnet.compact.compressed.wasm on-runtime-upgrade live --uri wss://rpc3.paseo.popnetwork.xyz:443
```

- [ ] (Advised) Runtime upgrade on a local **fork** of Pop Network.

_More instructions around using chopsticks for this can be found in [.chopsticks direcotry](../../.chopsticks)_

1. Launch the local fork:
```shell
    npx @acala-network/chopsticks@latest xcm -r ./.chopsticks/paseo.yml -p ./.chopsticks/testnet.yml
```
2. Do a runtime upgrade using the new runtime release -- can be found in `./target/release/wbuild/pop-runtime-testnet/pop_runtime_testnet.compact.compressed.wasm`.
3. Might be needed to trigger block production on Pop Network:
```shell
    websocat ws://localhost:8000
    {"jsonrpc":"2.0","id":2,"method":"dev_newBlock","params":[{"count":20}]}
```


## Release

> _Node release_

- [ ] New node release has been created: e.g. https://github.com/r0gue-io/pop-node/releases/tag/node-v0.2.0-alpha
    - Create a tag like `node-v<x.y.z>`.
    - Point to the previous node release to create a proper diff.
    - Provide a description and then release notes.

> _Runtime release_

- [ ] New runtime release has been created: e.g. https://github.com/r0gue-io/pop-node/releases/tag/runtime-v0.4.0-alpha
    - Create a tag like `runtime-v<x.y.z>`.
    - Point to the previous runtime release to create a proper diff.
    - Provide a description and then release notes.

- [ ] Create the authorization call data
    - < Edit with authorized runtime call data >
