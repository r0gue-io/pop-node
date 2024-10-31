# Pop API V0

The version `0` of Pop API features integrations with the following pallets:
- [Fungibles](#fungibles)


---
### FUNGIBLES
```rust
#[runtime::pallet_index(150)]
pub type Fungibles = fungibles::Pallet<Runtime>;
```
The fungibles pallet offers a streamlined interface for interacting with fungible tokens. The
goal is to provide a simplified, consistent API that adheres to standards in the smart contract
space.

For more details please refer to the [Fungibles API documentation](./fungibles/README.md).

Find examples in: [`../../examples/fungibles/`](../../examples/fungibles/).

The fungibles pallet can be found in [`pallets/api/src/fungibles/`](../../../pallets/api/src/fungibles/)
