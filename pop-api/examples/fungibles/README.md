# PSP22 Fungible Token with Pop API

This [ink!][ink] contract implements a [PSP22-compliant][psp22] fungible token by leveraging the [Pop API Fungibles][pop-api-fungibles]. Unlike typical token contracts, where the contract itself manages the token, tokens created by this contract are managed directly by Pop. This enables seamless integration and interoperability of the token across the Polkadot ecosystem and its applications.

As the creator of the token, the contract has permissions to mint and burn tokens, but it can only transfer and approve tokens on its own behalf and requires explicit approval to transfer tokens for other accounts. Instead of users interacting with the contract to handle their token approvals, they interact primarily with Pop’s runtime.

## Key benefits of using the Pop API

- The token operates live on the Pop Network, beyond just within the contract.
- Simplify token management with high-level interfaces to significantly reduce contract size and complexity.

[Learn more how Pop API works.](pop-api)

## Use Cases

This contract can serve a variety of purposes where owner-controlled token management is essential. Example use cases include:
- **DAO Token**: A DAO can use this contract to manage a governance token, with the DAO overseeing token issuance and removal based on governance decisions.
- **Staking and Rewards**: This contract supports minting tokens specifically for reward distribution.
- **Loyalty Programs**: Businesses or platforms can use this contract to issue loyalty points, with the owner managing token balances for users based on participation or purchases.

## Test with Pop Drink

Since this contract interacts directly with Pop’s runtime through the Pop API, it requires [Pop Drink](https://github.com/r0gue-io/pop-drink) for testing. See how the contract is tested in [tests](./tests.rs).

## Potential Improvements

- **Multiple owner management**: Instead of restricting ownership to a single `owner`, the contract could be designed to accommodate multiple owners.

## Support

Be part of our passionate community of Web3 builders. [Join our Telegram](https://t.me/onpopio)!

Feel free to raise issues if anything is unclear, you have ideas or want to contribute to Pop! Examples using the fungibles API are always welcome!

For any questions related to ink! you can also go to [Polkadot Stack Exchange](https://polkadot.stackexchange.com/) or
ask the [ink! community](https://t.me/inkathon/1).

[ink]: https://use.ink
[psp22]: https://github.com/inkdevhub/standards/blob/master/PSPs/psp-22.md
[pop-api]: https://github.com/r0gue-io/pop-node/tree/main/pop-api/
[pop-api-fungibles]: https://github.com/r0gue-io/pop-node/tree/main/pop-api/src/v0/fungibles
[pop-drink]: https://github.com/r0gue-io/pop-drink
