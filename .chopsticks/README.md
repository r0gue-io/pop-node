# Chopsticks

A brief guide on testing a runtime upgrade with [Chopsticks](https://github.com/AcalaNetwork/chopsticks/).

## Runtime Upgrade

1. Launch local network with forks of Pop and Paseo state:
    ```shell
    npx @acala-network/chopsticks@latest xcm -r ./.chopsticks/paseo.yml -p ./.chopsticks/testnet.yml
    ```
2. Authorise and apply the authorised runtime upgrade on the local Pop fork.
3. Build a block on the local relay chain to build a block (using [websocat](https://github.com/vi/websocat)).
     ```shell
     websocat ws://localhost:8001
     {"jsonrpc":"2.0","id":2,"method":"dev_newBlock","params":[{"count":1}]}
     ```
4. Build blocks on Pop to complete the upgrade:
    ```shell
    websocat ws://localhost:8000
    {"jsonrpc":"2.0","id":2,"method":"dev_newBlock","params":[{"count":10}]}
    ```
5. Verify that the runtime upgrade completed successfully.