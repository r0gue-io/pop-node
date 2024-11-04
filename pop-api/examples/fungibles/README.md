# PSP22 Fungible Token with Pop API


This [ink!][ink] contract shows a contract that allows interaction and management of a fungible token following the [PSP22 standard][psp22], utilizing the [Pop API Fungibles][pop-api-fungibles] feature. In this specific contract only the contract owner has a permission to call specific methods.

## Design Goals

- Token exists as an asset on the Pop Network, rather than being confined solely to the contract.
- Contract is the `origin` of the calls made by `pop-api`.
- Only the contract owner has a permission to call specific methods.

[Learn more how Pop API works.](/pop-api/README.md)

## Test with Pop DR!nk

Because the contract uses `pop-api` which calls to the runtime, it requires a special crate called [pop-drink][pop-drink] to test the contract. See how the contract is tested in [tests](./tests.rs).

## What can be improved?

- **Multiple owner management**: Instead of restricting ownership to a single `owner`, the contract could be designed to accommodate multiple owners.

## Use cases

This contract can be used in multiple different real world cases such as:

- **Governance Token in DAO**: The DAO uses this PSP22 contract to create a governance token, with DAO authority as owner, enabling actions like minting and burning tokens to manage member governance.
- **Staking Rewards**: This contract mints and burns tokens for a staking rewards program, allowing the staking authority to distribute rewards only to eligible participants.

## Support

Be part of our passionate community of Web3 builders. [Join our Telegram](https://t.me/onpopio)!

Feel free to raise issues if anything is unclear, have ideas or want to contribute to Pop! Examples using the fungibles API are always welcome!

For any questions related to ink! you can also go to [Polkadot Stack Exchange](https://polkadot.stackexchange.com/) or
ask the [ink! community](https://t.me/inkathon/1).

[ink]: https://use.ink
[psp22]: https://github.com/inkdevhub/standards/blob/master/PSPs/psp-22.md
[pop-api-fungibles]: https://github.com/r0gue-io/pop-node/tree/main/pop-api/src/v0/fungibles
[pop-drink]: https://github.com/r0gue-io/pop-drink
