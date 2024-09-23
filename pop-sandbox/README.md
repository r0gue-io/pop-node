# Pop Sandbox

Implementation of the [`pop_drink::Sandbox`](https://github.com/r0gue-io/pop-drink) struct for the Pop Network runtimes (located in `pop-node/runtime`) required for the quasi testing with `drink`.

In the context of quasi-testing with pop-drink, a sandbox refers to an isolated runtime environment that simulates the behavior of a full node, without requiring an actual node. It can emulate key processes (where runtime `pallets` are involved) such as block initialization, execution, and block finalization.

## Getting Started

### Installation

```toml
pop_drink = { version = "1.0.0",  package = "pop-drink" }
```

### Import Sandbox for the specific runtime

- For `devnet` runtime

Implementation of the sandbox runtime environment for `devnet` runtime located in `pop-node/runtime/devnet`

```rs
use pop_sandbox::DevnetSandbox;
```

- For `testnet` runtime

Implementation of the sandbox runtime environment for `testnet` runtime located in `pop-node/runtime/testnet`

```rs
use pop_sandbox::TestnetSandbox;
```

- For `mainnet` runtime

Implementation of the sandbox runtime environment for `mainnet` runtime located in `pop-node/runtime/mainnet`

```rs
use pop_sandbox::MainnetSandbox;
```

### Setup test environment for your contract

Below is an example for the contract testing with `pop_drink` and `pop_sandbox` for `devnet` environment using `DevnetSandbox`.

```rs
use pop_drink::session::Session;
use pop_sandbox::DevnetSandbox as Sandbox;

#[drink::contract_bundle_provider]
enum BundleProvider {}

#[drink::test(sandbox = Sandbox)]
fn test(mut session: Session) {
 // Your test case
}
```

## Examples

Please find more examples of `pop_drink` tests in the [`pop_drink/examples`](https://github.com/r0gue-io/pop-drink/tree/main/examples).
