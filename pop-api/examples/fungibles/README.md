# PSP22 Example Contract with Pop Fungibles API

This contract is an example implementation of the [PSP22 standard](https://github.com/inkdevhub/standards/blob/master/PSPs/psp-22.md), utilizing the Pop API Fungibles module.

- This contract provides core functionalities to manage a PSP22 standard token that exists as an asset on the Pop Network, rather than being confined solely to the contract.
- Only the contract owner has a permission to call specific methods.

This design choice is due to the fact that `pop-api` invokes runtime calls with the contract itself as the `origin`. For examples, methods like `transfer` and `transfer_from` will operate on the contract's own account balance, not on the balance of the caller.

To learn more about Pop API and how Pop API works under the hood: [Read here](/pop-api/README.md)

## Security risks prevention

To prevent potential misuse of contract methods by malicious actors—such as using `approve` to grant unauthorized transfer permissions—the contract restricts access exclusively to the `owner` (the account that instantiated the contract).

```rs
self.ensure_owner()?;
```

Only the `owner` is permitted to call these methods, ensuring the contract’s balance remains secure. The owner of the contract can be updated by an existing `owner` by calling the method `transfer_ownership`.

## What can be improved?

- Instead of restricting ownership to a single `owner`, the contract could be designed to accommodate multiple owners. Therefore, the `transfer_ownership` is also need to be updated to support this feature.

## Use cases

This contract can be used in multiple different real world cases such as:

- **Use case for a governance token within a [DAO (Decentralized Autonomous Organization)](https://www.investopedia.com/tech/what-dao/)**: The DAO contract performs cross-contract calls to this PSP22 example contract to create a new token, with the DAO authority as the token’s owner. This allows the DAO authority to manage governance tokens for its members by performing actions such as mint and burn.

- **Staking Rewards Program**: This contract can be used to issue rewards in a staking program, where users lock tokens for network security or liquidity. The staking contract calls this PSP22 contract to mint and burn tokens, allowing the staking authority to manage rewards and ensure distribution only to eligible participants.
