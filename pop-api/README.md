## Pop API

The `pop-api` crate provides a high-level interface that allows smart contracts to seamlessly interact with Pop, a
blockchain built to power innovative and impactful solutions on Polkadot. Designed for stability, simplicity and
efficiency, the api abstracts away the complexities of the runtime, enabling developers to focus on building powerful
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
    - Planned:
        - Non Fungibles (Dec)
        - Messaging; ISMP and XCM rails (Jan - Feb)
        - Sponsorship (TBD)

### Getting Started

Using the api in your ink! smart contract is as easy as adding the `pop-api` crate in your `Cargo.toml`:

```toml
pop-api = { git = "https://github.com/r0gue-io/pop-node", default-features = false }
```

and importing it within the contract source:

```rust
use pop_api::*;
```

Check out the ink! smart contract [examples](./example) using the api.

### Learn more

The true strength of the api lies in the Pop runtime, where a single, unified chain extension provides flexible and
efficient access to all runtime features, while specialized API modules deliver stable, intuitive interfaces for
developers. Together, these elements make the api a powerful tool for creating decentralized applications on Polkadot.

Want to explore how it all works? Check out:

- [Chain Extension Implementation](../extension)
    - [Devnet Configuration](../runtime/devnet/src/config/api)
- [API Modules](../pallets/api)

