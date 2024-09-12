# Pop Sandbox

Implementation of the `pop_drink::Sandbox` struct for the Pop Network runtimes required for the quasi testing with `drink`.

## Getting Started

### Installation

```toml
pop_drink = { version = "1.0.0",  package = "pop-drink" }
```

### Import Sandbox for the specific runtime

For mainnet

```rs
use pop_sandbox::MainnetSandbox;
```

For devnet

```rs
use pop_sandbox::DevnetSandbox;
```

For testnet

```rs
use pop_sandbox::TestnetSandbox;
```

### Setup test environment for your contract

```rs
use drink::session::Session;
use pop_sandbox::DevnetSandbox as Sandbox;

#[drink::contract_bundle_provider]
enum BundleProvider {}

#[drink::test(sandbox = Sandbox)]
fn test(mut session: Session) {
 // Your test case
}
```
