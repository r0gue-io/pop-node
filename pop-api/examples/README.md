## Pop API Examples

The example contracts demonstrate how to use the `pop_api` interface with [ink!](https://use.ink).

## Warning

The available contracts are *examples* demonstrating usage of Pop's smart contract API. They are neither audited nor
endorsed for production use. Do **not** rely on them to keep anything of value secure.

## Development

### Prerequisite

[Pop CLI](https://github.com/r0gue-io/pop-cli) installed.

### Launching a local Pop Network

The example contracts only work with Pop Network runtimes due to their usage of Pop's smart contract API. To run
`pop-node` locally, you can use the following command:

```bash
pop up network -f ./networks/devnet.toml
```

The output should provide a command for following the logs emitted from `pop-node`, which can be useful for debugging:

```
 logs: tail -f /var/folders/mr/gvb9gkhx58x2mxbpc6dw77ph0000gn/T/zombie-d9983e5b-fc70-478d-b943-d920a659d308/pop/pop.log
```

> 📚 See the full guide to launching a chain
> locally [here](https://learn.onpop.io/appchains/guides/launch-a-chain/running-your-parachain).

### Build a contract

Run the below command to build the contract:

```bash
pop build -r
```

This builds the contract in release mode.

### Upload and instantiate a contract

Upload your contract to the local instance of Pop Network you launched above with the following command, replacing the
url with that shown in the output of the `pop up network` command.

```bash
pop up contract \
    --url=ws://127.0.0.1:9944 \
    # The value provided at instantiation (via `payable`).
    --value 100000000000 \
    # Using Alice as the contract owner, you can provide `--use-wallet` to sign with your own wallet.
    --suri //Alice \
    # Provide the constructor args
    --args ...
```

## Support

Be part of our passionate community of Web3 builders. [Join our Telegram](https://t.me/onpopio)!

Feel free to raise issues if anything is unclear, you have ideas or want to contribute to Pop! Examples using the
API are always welcome!

For any questions related to ink! you can also go to [Polkadot Stack Exchange](https://polkadot.stackexchange.com/) or
ask the [ink! community](https://t.me/inkathon/1).

- [Learn more about ink! smart contract language](https://use.ink).
- Learn more about [Pop API](https://github.com/r0gue-io/pop-node/tree/main/pop-api/).
