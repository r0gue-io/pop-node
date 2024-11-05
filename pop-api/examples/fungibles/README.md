# PSP22 Fungible Token with Pop API

This [ink!][ink] contract demonstrates a [PSP22-compliant][psp22] fungible token utilizing the [Pop API Fungibles][pop-api-fungibles]. Unlike typical token contracts, where the contract itself manages the token, tokens created by this contract are managed directly by Pop. Instead of users interacting with the contract to handle their tokens, they interact with Pop’s runtime.

As the token owner, the contract has permissions to mint and burn tokens, but it can only transfer and approve tokens on its own behalf and requires explicit approval to transfer tokens for other accounts. This structure enables seamless integration and interoperability across the Polkadot ecosystem and its applications.

## Use Cases

This contract can serve a variety of purposes where owner-controlled token management is essential. Example use cases include:
- **DAO Token**: A DAO can use this contract to manage a governance token, with the DAO overseeing token issuance and removal based on governance decisions.
- **Staking and Rewards**: This contract supports minting tokens specifically for reward distribution.
- **Loyalty Programs**: Businesses or platforms can use this contract to issue loyalty points, with the owner managing token balances for users based on participation or purchases.ints and burns tokens for a staking rewards program, allowing the staking authority to distribute rewards only to eligible participants.

## Test with Pop Drink

Since this contract interacts directly with Pop’s runtime through the Pop API, it requires [Pop Drink](https://github.com/r0gue-io/pop-drink) for testing. See how the contract is tested in [tests](./tests.rs).

## Potential Improvements

- **Multiple owner management**: Instead of restricting ownership to a single `owner`, the contract could be designed to accommodate multiple owners.

## Support

Be part of our passionate community of Web3 builders. [Join our Telegram](https://t.me/onpopio)!

Feel free to raise issues if anything is unclear, have ideas or want to contribute to Pop! Examples using the fungibles API are always welcome!

For any questions related to ink! you can also go to [Polkadot Stack Exchange](https://polkadot.stackexchange.com/) or
ask the [ink! community](https://t.me/inkathon/1).

[ink]: https://use.ink
[psp22]: https://github.com/inkdevhub/standards/blob/master/PSPs/psp-22.md
[pop-api-fungibles]: https://github.com/r0gue-io/pop-node/tree/main/pop-api/src/v0/fungibles
[pop-drink]: https://github.com/r0gue-io/pop-drink
