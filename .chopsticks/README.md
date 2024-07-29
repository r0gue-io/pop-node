# Chopsticks

This folder contains the config file to run [chopsticks](https://github.com/AcalaNetwork/chopsticks/tree/master) against
Pop Network on Paseo.

## Install and run

1. Clone repository with submodules ([smoldot](https://github.com/paritytech/smoldot)). It is expected that
   the `chopsticks` folder is in the same directory as the `pop-node` directory.
    ```shell
    git clone --recurse-submodules https://github.com/AcalaNetwork/chopsticks.git
    ```
2. Go to `chopsticks` repo.
    ```shell
    cd chopsticks
    ```
3. Install deps.
    ```shell
    yarn
    ```
4. Build wasm. Please do not use IDE's built-in tools to build wasm.
    ```shell
    yarn build-wasm
    ```
5. Start chopsticks (assuming the `pop-node` folder is called `pop-node`)
    ```shell
    npx @acala-network/chopsticks@latest --config=../pop-node/.chopsticks/dev.yml
    ```

## Runtime Upgrade

1. Build a new runtime using the `Build Deterministic Runtimes` GitHub Action.
2. Start chopsticks forks of Pop Network testnet and Paseo relay chain state:
    ```shell
    npx @acala-network/chopsticks@latest xcm -r ../pop-node/.chopsticks/paseo.yml -p ../pop-node/.chopsticks/test.yml
    ```
3. Authorize and enact an upgrade following the [guide](../docs/runtime-upgrade.md).
4. Build a block on the relay chain to confirm the upgrade:
   ```shell
   websocat ws://localhost:8001
   ```
   ```shell
   {"jsonrpc":"2.0","id":2,"method":"dev_newBlock","params":[{"count":1}]}
   ```
5. Build blocks on Pop Network testnet to complete the upgrade (e.g. until you see related events in PJS):
   ```shell
   websocat ws://localhost:8000
   ```
   ```shell
   {"jsonrpc":"2.0","id":2,"method":"dev_newBlock","params":[{"count":2}]}
   ```
6. Verify that the runtime upgrade completed successfully by checking the resulting version.
