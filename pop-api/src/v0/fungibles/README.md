# Fungibles API

The `fungibles` module provides an api for interacting and managing fungible tokens.

It includes the following interfaces:

1. PSP-22
2. PSP-22 Metadata
3. Management
4. PSP-22 Mintable & Burnable

To use it in your contract add the `fungibles` feature to the `pop-api` dependency.

```toml
# Cargo.toml
pop-api = { git = "https://github.com/r0gue-io/pop-node", default-features = false, features = [ "fungibles" ] }
```

Check out the [examples](../../examples/fungibles/) to learn how you can use the fungibles api.