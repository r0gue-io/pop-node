# Non-Fungibles Contract Example

This smart contract demonstrates how to manage NFTs using the [
`pop_api::nonfungibles`](../../src/v0/nonfungibles) interface with [ink!](https://use.ink).

### Upload and instantiate a contract

See [examples](../README.md#development) for instructions on getting started and then upload your contract with the
following command.

```bash
pop up contract \
    --url=ws://127.0.0.1:9944 \
    # The value provided at instantiation (via `payable`) to reserve the deposit for the collection.
    --value 100000000000 \
    # Using Alice as the contract owner, you can provide `--use-wallet` to sign with your own wallet.
    --suri //Alice \
    # Provide the max supply
    --args 1000
```

## Support

Be part of our passionate community of Web3 builders. [Join our Telegram](https://t.me/onpopio)!

Feel free to raise issues if anything is unclear, you have ideas or want to contribute to Pop! Examples using the
non-fungibles API are always welcome!

For any questions related to ink! you can also go to [Polkadot Stack Exchange](https://polkadot.stackexchange.com/) or
ask the [ink! community](https://t.me/inkathon/1).

[ink]: https://use.ink

[psp34]: https://github.com/inkdevhub/standards/blob/master/PSPs/psp-34.md

[pop-api]: https://github.com/r0gue-io/pop-node/tree/main/pop-api/

[pop-api-nonfungibles]: https://github.com/r0gue-io/pop-node/tree/main/pop-api/src/v0/nonfungibles

[pop-drink]: https://github.com/r0gue-io/pop-drink