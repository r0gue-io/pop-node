# Pop API

## Examples
ink! smart contract examples using the Pop API
- [balance-transfer](./examples/balance-transfer/)
- [NFTs](./examples/nfts/)
- [place-spot-order](./examples/place-spot-order/)
- [read-runtime-state](./examples/read-runtime-state/)

## What is the Pop API?

One of the core value propositions of Pop Network is to enable smart contracts to easily access the power of Polkadot. As such, the Pop API was built to enable smart contracts to easily utilize the functionality provided by the Pop Network parachain runtime.

Substrate already exposes a Runtime API which is typically associated with the “outer node” calling into the runtime via RPC. Pop Network extends this concept in a direction that makes it usable by smart contracts, allowing untrusted contracts to make use of carefully exposed trusted functionality available within the runtime - this is the Pop API.

The Pop API is designed to be:
- **Stable**: contracts are generally immutable and therefore need a stable interface, especially as the runtime evolves over time.
- **Future Proof**: technologies improve, so the Pop API implementation is adaptable to new approaches without affecting the interface. The Pop API implementation also selects the optimal approach to realize a use case from a range of possible technical implementations.
- **Versioned**: not every future technology can be retro-fitted, so the API is versioned.
- **Simple**: providing an abstraction layer over runtime-level features. Hand-rolled with love, Pop API is easy to use for smart contract development.

The Pop API consists of three main parts:
- the Pop API ink! library
- the Pop API runtime code (chain extension)
- shared primitive types between the Pop API ink! library and the Pop API runtime code

Let's go over the flow.

## The Pop API ink! Library
Everything in [`pop-api`](./src/) **is the** Pop API ink! library.

So when the ink! smart contract wants to use the Pop API library, it can simply have a line like:
```rust
use pop_api::nfts::*;
```

## The Glue

Certain types are shared between the ink! library portion of the Pop API and the runtime portion of the Pop API. These types can be found in [`pop_primitives`](../primitives/src/), outside the [`pop-api`](./src/) folder.

## The Entry into the Pop API

When we use the Pop API in our smart contract like so:
```rust
use pop_api::nfts::*;
mint(collection_id, item_id, receiver)?;
```

This will call the Pop API `mint` function in the [./src/v0/nfts.rs](./src/v0/nfts.rs) file, which is a wrapper to `dispatch` a `Runtime::Call` to the NFTs pallet's mint function. This is how most of the Pop API is built. This abstraction allows for creating a developer-friendly API over runtime level features such as calling pallets, reading state, and cross-chain interactions. All Pop API functionality can be found in [./src/v0/](./src/v0/) which is the current version of the Pop API.


## Dispatching to the runtime ([./src/lib.rs](./src/lib.rs))

### `PopApi` 
The `PopApi` trait is an ink! chain extension trait with three functions:
- `dispatch()`
- `read_state()`
- `send_xcm()`

These are the workhorse functions of the Pop API. Through these functions all the interactions between the ink! smart contract and the runtime are carried out. So in our example above, the `mint` function in [nfts.rs](./src/v0/nfts.rs) calls a `dispatch`, this `dispatch` is defined here in the [lib.rs](./src/lib.rs) file. It is what calls into the runtime chain extension.

> Notice how each function is assigned a function ID e.g. `#[ink(function = 1)]`. This will play a role later when we cover the runtime portion of the chain extension.

### `PopApiError`
When Pop API calls the runtime, it will either receive a successful result or an encoded error. `PopApiError` translates the encoded error into the appropriate module error according to the index that the pallet has been configured to.

## The Pop API Chain Extension

So we have covered how the ink! Pop API library calls the chain extension. But where is the chain extension actually defined? In the Pop Network runtime.

Chain extensions "extend" a runtime. We can find the `PopApiExtension` chain extension in [extension.rs](../runtime/devnet/src/extensions.rs). The `PopApiExtension` chain extension is matching based on the function IDs that we defined on the ink! side of the Pop API. The chain extension here will execute the appropriate functions e.g. `dispatch` or `read_state`. These functions are defined in this file as well and interact directly with the Pop Network runtime.

If you would like to see the whole flow, checkout the end-to-end tests in [extensions.rs](../runtime/devnet/src/extensions.rs) file e.g. `dispatch_nfts_mint_from_contract_works()`
