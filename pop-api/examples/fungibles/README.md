# PSP22 Fungible Token with Pop API

PSP22 is a fungible token standard for WebAssembly smart contracts running on blockchains based on the [Substrate][substrate] framework. It is an equivalent of Ethereum's [ERC-20][erc20]. The definition of the PSP22 standard can be found [here][psp22].

This repository contains a simple, minimal implementation of the PSP22 token in [ink!][ink] smart contract programming language (EDSL based on Rust), utilizing the [Pop API Fungibles][pop-api-fungibles] feature.

> [!IMPORTANT] > This version of the PSP22 contract is compatible with ink! 5.

## Design Goals

- Token exists as an asset on the Pop Network via [pallet-assets][pallet-assets], rather than being confined solely to the contract.
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

Feel free to raise issues if anything is unclear, have ideas or want to contribute to Pop!

For any questions related to ink! you can also go to [Polkadot Stack Exchange](https://polkadot.stackexchange.com/) or
ask the [ink! community](https://t.me/inkathon/1).

[ink]: https://use.ink
[substrate]: https://substrate.io
[erc20]: https://ethereum.org/en/developers/docs/standards/tokens/erc-20/
[psp22]: https://github.com/inkdevhub/standards/blob/master/PSPs/psp-22.md
[pop-api-fungibles]: https://github.com/r0gue-io/pop-node/tree/main/pop-api/src/v0/fungibles
[pallet-assets]: https://crates.io/crates/pallet-assets
[pop-drink]: https://github.com/r0gue-io/pop-drink
