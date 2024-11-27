## Pop API

The `pop-api` crate provides a high-level interface that allows smart contracts to seamlessly interact with Pop, a
blockchain built to power innovative and impactful solutions on Polkadot. Designed for stability, simplicity, and
efficiency, the API abstracts away the complexities of the runtime, enabling developers to focus on building powerful
applications rather than managing intricate blockchain details.

### Design Goals

- **Simple**: enhance the developer experience by abstracting away the complexities of runtime functionality, making it
  easy for developers to build advanced applications.
- **Versioned**: offer a stable, versioned interface that ensures smart contracts stay compatible as the runtime
  evolves, enabling seamless integration of new features without disrupting existing contracts.
- **Efficient**: optimise for minimal contract size, having the lowest possible deployment and execution costs.

### Key Features

- **Versioned Interface**: Provides backward compatibility, ensuring that existing contract functionality remains stable
  as new features are added to the runtime.
- **Error Handling**: Supports rich, versioned error types, enabling contracts to receive and interpret any runtime
  error, making troubleshooting and development easier.
- **Use Cases**:
    - [Fungibles](./src/v0/fungibles/README.md): Interacting and managing fungible tokens.
    - In development:
        - Non Fungibles: Interacting and managing non fungible tokens.
        - Messaging: Cross chain rails for interaction with other chains using ISMP & XCM.
        - Sponsorship: Allowing smart contracts to sponsor transactions.
        - Incentives: Incentivise smart contracts by sharing chain revenue.

### Getting Started

Using the API in your ink! smart contract is as easy as adding the `pop-api` crate in your `Cargo.toml`:

```toml
pop-api = { git = "https://github.com/r0gue-io/pop-node", default-features = false }
```

and importing it within the contract source:

```rust
use pop_api::*;
```

Check out the ink! smart contract [examples](./example) using the API.

### Learn more

The true strength of the API lies in the Pop runtime, where a single, unified chain extension provides flexible and
efficient access to all runtime features. Go check out the [extension](../extension) if you want to learn more!

### Support

Be part of our passionate community of Web3 builders. [Join our Telegram](https://t.me/onpopio)!

Feel free to raise issues if anything is unclear, if you have ideas or want to contribute to Pop!

For any questions related to ink! you can also go to [Polkadot Stack Exchange](https://polkadot.stackexchange.com/) or
ask the [ink! community](https://t.me/inkathon/).
