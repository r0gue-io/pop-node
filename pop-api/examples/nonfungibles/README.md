# Nonfungibles Contract Example

This smart contract demonstrates how to manage NFTs using the [`pop_api::nonfungibles`](https://docs.rs/pop-api/latest/pop_api/nonfungibles/) interface with [ink!](https://use.ink).

## Development

### Prerequisite

Below guides use a tool called [pop-cli](https://github.com/r0gue-io/pop-cli). You can install Pop CLI from crates.io:

```bash
cargo install --force --locked pop-cli
```

> ℹ️ Pop CLI requires Rust 1.81 or later.

[Learn more about Pop CLI here](https://github.com/r0gue-io/pop-cli).

### Launching a local Pop Network

The contract only works with Pop Network runtimes, to run `pop-node` on local, you can use the following command:

> 📚 See the full guide to launching a chain locally [here](https://learn.onpop.io/appchains/guides/launch-a-chain/running-your-parachain).

```bash
pop up network -f ./networks/devnet.toml
```

### Build a contract

Run the below command to build the contract:

```bash
pop build -r
```

This builds the contract in release mode.

### Upload and instantiate a contract

If you are following the exact guide for launching the chain, so your local Pop Network is running on port `9944`, you can upload and instantiate the contract using the following command:

```bash
pop up contract \
    --url=ws://127.0.0.1:9944 \
    # The value provided at instantiation (via `payable`) to reserve the deposit for the collection.
    --value 100000000000 \
    # Using Alice as the contract owner, you can provide `--use-wallet` to sign with your own wallet.
    --suri //Alice \
    # Provide the max supply and the mint price
    --args 1000 100
```

## Documentation

To generate the documentation, run the following command in the root directory of the contract:

```bash
cargo doc --open
```

## Support

Be part of our passionate community of Web3 builders. [Join our Telegram](https://t.me/onpopio)!

Feel free to raise issues if anything is unclear, you have ideas or want to contribute to Pop! Examples using the fungibles API are always welcome!

For any questions related to ink! you can also go to [Polkadot Stack Exchange](https://polkadot.stackexchange.com/) or
ask the [ink! community](https://t.me/inkathon/1).

- [Learn more about ink! smart contract language](https://use.ink).
- About [PSP34](https://github.com/inkdevhub/standards/blob/master/PSPs/psp-34.md) standard.
- Learn more about [Pop API](https://github.com/r0gue-io/pop-node/tree/main/pop-api/) and [Nonfungibles feature](https://github.com/r0gue-io/pop-node/tree/main/pop-api/).
