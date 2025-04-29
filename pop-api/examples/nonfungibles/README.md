# Nonfungibles Contract Example

This smart contract demonstrates how to manage NFTs using the [`pop_api::nonfungibles`](https://docs.rs/pop-api/latest/pop_api/nonfungibles/) interface with [ink!](https://use.ink).

## Features

- Create an NFT collection.
- Mint NFTs.
- Transfer NFTs owned by the contract.
- Burn NFTs owned by the contract.
- Query ownership, balances, and total supply.
- Destroy the NFT collection and self-destruct the contract.

## Functions

| Function | Description |
| :--- | :--- |
| `new(max_supply, price)` | Deploys the contract and creates a new NFT collection with a mint price and maximum supply. |
| `collection_id()` | Returns the collection ID managed by this contract. |
| `balance_of(owner)` | Returns the number of NFTs an owner has. |
| `owner_of(item)` | Returns the owner of a specific item. |
| `total_supply()` | Returns the number of minted items. |
| `mint(to, item, witness)` | Mints a new item to an account. |
| `burn(item)` | Burns an NFT owned by the contract. |
| `transfer(to, item)` | Transfers an NFT from the contract to another account. |
| `destroy(destroy_witness)` | Destroys the collection and self-destructs the contract. |

## Notes

- The contract must be deployed as **payable** to handle deposits.
- Deposits are required for creating collections and minting NFTs.
- Only the original deployer (owner) can call `destroy` to clean up and reclaim deposits.

## Support

Be part of our passionate community of Web3 builders. [Join our Telegram](https://t.me/onpopio)!

Feel free to raise issues if anything is unclear, you have ideas or want to contribute to Pop! Examples using the fungibles API are always welcome!

For any questions related to ink! you can also go to [Polkadot Stack Exchange](https://polkadot.stackexchange.com/) or
ask the [ink! community](https://t.me/inkathon/1).

[ink]: https://use.ink
[psp34]: https://github.com/inkdevhub/standards/blob/master/PSPs/psp-34.md
[pop-api]: https://github.com/r0gue-io/pop-node/tree/main/pop-api/
[pop-api-nonfungibles]: https://github.com/r0gue-io/pop-node/tree/main/pop-api/src/v0/nonfungibles
