## Pop Chain Extension
_A Chain Extension for Polkadot SDK Runtimes_

The pop-chain-extension crate provides a chain extension designed to be integrated with any Polkadot SDK runtime. Instead of being limited to a single pallet, the extension offers a flexible solution that can cater to any feature exposed by the runtime. It provides functionality to dispatch runtime calls or query runtime state from smart contracts.

### Key Features
**Precise Weight Charging**: Handles accurate weight charging ensuring secure execution.

**I/O handling**:
- **Error Handling**: Supports flexible error processing returned by the runtime and send back to the smart contracts.

- **Input Handling**: Supports flexible input processing for interacting with the runtime.

_Motivation: versioning, enabling contracts to interact with different versions of the runtime, providing backward compatibility._

**Rich Logging for Debugging**: Provides detailed logging capabilities with customizable log targets, aiding developers in debugging and monitoring contract interactions with the runtime.

### Learn more
