## Messaging API

The `messaging` module provides an api for cross-chain interactions.

It includes the following interfaces:

1. `Messaging`: a general interface for cross-chain messaging operations.
2. `Ismp`: a streamlined interface for messaging using the Interoperable State Machine Protocol.
3. `IsmpCallback`: as above, but with additional callback functionality.
4. `Xcm`: a streamlined interface for messaging using Polkadot's Cross-Consensus Messaging (XCM).
5. `XcmCallback`: as above, but with additional callback functionality.

To use it in your contract add the `messaging` feature to the `pop-api` dependency.

```toml
# Cargo.toml
pop-api = { git = "https://github.com/r0gue-io/pop-node", default-features = false, features = [ "messaging" ] }
```

Check out the [examples](../../examples/messaging/) to learn how you can use the messaging api.
