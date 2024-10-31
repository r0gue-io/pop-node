# Pop API

_A stable runtime interface for ink! smart contracts that elevates the experience of building Web3 applications._

---

## What is the Pop API?

One of the core value propositions of Pop Network is to enable smart contracts to easily access the power of Polkadot. As such, the Pop API was built to enable smart contracts to easily utilize the functionality provided by the Pop Network parachain runtime.

## Versions

- [V0](./src/v0/README.md)

## Examples
ink! smart contract examples using the Pop API

- [fungibles](./examples/fungibles/)
- [read-runtime-state](./examples/read-runtime-state/)

---

## Design 

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
use pop_api::fungibles::{self as api};
```

## The Glue

Certain types are shared between the ink! library portion of the Pop API and the runtime portion of the Pop API. These types can be found in [`pop_primitives`](../primitives/src/), outside the [`pop-api`](./src/) folder.

## The Entry into the Pop API

When we use the Pop API in our smart contract like so:
```rust
use pop_api::fungibles::{self as api};
// -- snip --
#[ink(message)]
pub fn transfer(&mut self, token: TokenId, to: AccountId, value: Balance) -> Result<()> {
    // Use of Pop API to call into the runtime to transfer some fungible assets.
    api::transfer(token, to, value)
}
```

This will call the Pop API `transfer` function in the [./src/v0/fungibles/mod.rs](./src/v0/fungibles/mod.rs) file, which is a wrapper to `dispatch` a `Runtime::Call` to the assets pallet's transfer function. This is how most of the Pop API is built. This abstraction allows for creating a developer-friendly API over runtime level features such as calling pallets, reading state, and cross-chain interactions. All Pop API versions can be found in [pop-api/src/](./src/).


## Dispatching to the runtime ([./src/lib.rs](./src/lib.rs))

### `PopApi`
The `PopApi` is an ink! [`ChainExtensionMethod`](https://docs.rs/ink_env/5.0.0/ink_env/chain_extension/struct.ChainExtensionMethod.html) instance used to derive the execution of the different calls a smart contract can do into the runtime.

Its purpose its two fold, constructing the runtime calls that are going to be executed by the chain extension and handling the information returned.

The calls are built out of the following information:
- The `function` Id: Identifies the specific function within the chain extension.
- The API `version`: This byte allows the runtime to distinguish between different API versions, ensuring that older contracts call the correct, version-specific implementation..
- A `module` index: Identifies the pallet responsible for handling the call.
- A `dispatchable` index: Indicates the specific dispatchable or state read function within the pallet.

Multiple **functions** can be implemented for the chain extension, so whenever something needs to be added or changed, a new function will have to be implemented.
`DISPATCH` and `READ_STATE` functions are the workhorse functions of the Pop API.
Through these functions all the interactions between the ink! smart contract and the runtime are carried out.

By embedding the **version** directly into the encoding scheme, the runtime can manage different versions of dispatch calls and queries, ensuring that both legacy and new contracts function as intended, even as the underlying system evolves. This structure provides the flexibility needed to support ongoing improvements and changes in the runtime without disrupting existing smart contracts.

So in our example above, when the `trasnfer` function in [./src/v0/fungibles/mod.rs](./src/v0/fungibles/mod.rs) is called, the following is constructed `u32::from_le_bytes([DISPATCH, 0, FUNGIBLES, TRANSFER])` to be executed by the runtime chain extension.


### `StatusCode`
When Pop API calls the runtime, it will either receive a successful result or an encoded error. `StatusCode` translates the encoded error into the appropriate module error according to the index that the pallet has been configured to.

## The Pop API Chain Extension

So we have covered how the ink! Pop API library calls the chain extension. But where is the chain extension actually defined? In the Pop Network runtime.

Chain extensions "extend" a runtime. We can find the `PopApiExtension` chain extension in [extension.rs](../runtime/devnet/src/extensions.rs). The `PopApiExtension` chain extension is matching based on the function IDs that we defined on the ink! side of the Pop API. The chain extension here will execute the appropriate functions e.g. `dispatch` or `read_state`. These functions are defined in this file as well and interact directly with the Pop Network runtime.

`PopApiExtension` implements the different **functions** that can be called by the API. Two functions are provided:

- `DISPATCH`:
The Dispatch function decodes the received bytes into a `RuntimeCall`, optionally processing them as needed (versioning). It filters out any calls that are not explicitly permitted, then charges the appropriate weight, dispatches the call, and adjusts the weight accordingly before returning the result of the dispatch call.

- `READ_STATE`:
The ReadState function decodes the received bytes into a `Readable` type, with optional processing (versioning). It filters out any unauthorised state reads, charges the appropriate weight, executes the state read, optionally converts the result (versioning), and finally returns the result.

If you would like to see the whole flow, checkout the integration tests in [pop-api/integration-tests](./integration-tests/) e.g. [`instantiate_and_create_fungible_works()`](./integration-tests/src/fungibles/mod.rs).
