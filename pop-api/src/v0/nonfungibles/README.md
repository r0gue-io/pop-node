## Non-Fungibles API

The `nonfungibles` module provides an api for interacting and managing non-fungible tokens.

It includes the following interfaces:

1. PSP-34
2. PSP-34 Metadata
3. Management
4. PSP-34 Mintable & Burnable

To use it in your contract add the `nonfungibles` feature to the `pop-api` dependency.

```toml
# Cargo.toml
pop-api = { git = "https://github.com/r0gue-io/pop-node", default-features = false, features = [ "nonfungibles" ] }
```

Check out the [examples](../../examples/nonfungibles/) to learn how you can use the non-fungibles api.
