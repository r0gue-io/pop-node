# Pop API V0

The version `0` of Pop API features integrations with the following pallets:
- [Fungibles](#fungibles)

Each pallet integration can be included individually with its own feature, this way only the necessary dependencies will be compiled into the contract.

---
### FUNGIBLES
| pop-api feature |  pallet-index  |
|:---------------:|:--------------:|
|   `fungibles`   |     `150`      |

The fungibles pallet offers a streamlined interface for interacting with fungible tokens. The
goal is to provide a simplified, consistent API that adheres to standards in the smart contract
space.

To use the `Fungibles` API the `fungibles` features needs to be included in the pop-api crate dependency.
```toml
# Cargo.toml
pop-api = { git = "https://github.com/r0gue-io/pop-node", default-features = false, features = [ "fungibles" ] }
```

For more details please refer to the [Fungibles API documentation](./fungibles/README.md).

Find examples in: [`../../examples/fungibles/`](../../examples/fungibles/).

The fungibles pallet can be found in [`pallets/api/src/fungibles/`](../../../pallets/api/src/fungibles/).
