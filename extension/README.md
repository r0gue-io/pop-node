## Pop Chain Extension

_A Chain Extension for Polkadot SDK Runtimes_

The pop-chain-extension crate introduces a versatile chain extension for seamless integration with any Polkadot SDK runtime. Unlike being confined to a single pallet, it offers a flexible, runtime-agnostic solution that adapts to any feature exposed by the runtime. Optimized for smart contracts, it enables backward-compatible dispatch of runtime calls and state queries, ensuring robust and future-proof functionality.

### Key Features

**Precise Weight Charging**: Handles accurate weight charging ensuring secure execution.

**I/O handling**:

_Motivation: versioning, enabling contracts to interact with different versions of the runtime, providing backward compatibility._

- **Error Handling**: Supports flexible error processing returned by the runtime and send back to the smart contracts.

- **Input Handling**: Supports flexible input processing for interacting with the runtime.

**Rich Logging for Debugging**: Provides detailed logging capabilities with customizable log targets, aiding developers in debugging and monitoring contract interactions with the runtime.

### Learn more

The Pop Chain Extension is integrated into Pop's runtime, serving as the foundation for the [Pop API ink! library](../pop-api).

Check out how the extension is [configured in the Testnet runtime](../runtime/devnet/src/config/api) and find out how the api modules come into play as well!

### Support

Be part of our passionate community of Web3 builders. [Join our Telegram](https://t.me/onpopio)!

Feel free to raise issues if anything is unclear, if you have ideas or want to contribute to Pop!

For any questions related to ink! you can also go to [Polkadot Stack Exchange](https://polkadot.stackexchange.com/) or
ask the [ink! community](https://t.me/inkathon/).
