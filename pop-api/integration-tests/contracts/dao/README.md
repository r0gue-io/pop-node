# DAO with NFT Verification

**Important Note**: This code was used for a conference presentation, and hence has some sharp edges. It is not secure code.
For example, the NFT verification has a vulnerability where verifying the NFT on Asset Hub (AH) is insecure. This is because
ISMP requires finalization on the Polkadot relay chain, which means AH will be a few blocks ahead and the NFT may not even belong
to the user anymore.

# Guide for the Multichain DAO Smart Contract

This guide provides an overview of an Ink! smart contract named `Dao`, designed to showcase cross-chain interactions using a hybrid messaging API that combines XCM and ISMP. The contract enables users to register NFTs (Non-Fungible Tokens) from another parachain, verify their ownership, and mint corresponding NFTs on the local chain. Additionally, it supports executing transactions on the other parachain via XCM. Below, you'll find detailed instructions on how to deploy and interact with this contract, along with explanations of its key functionalities.

---

<aside>
ℹ️

This a proof-of-concept presented at Sub0

- Contract: [https://github.com/r0gue-io/pop-node/blob/daan/sub0/pop-api/integration-tests/contracts/dao/lib.rs](https://github.com/r0gue-io/pop-node/blob/daan/sub0/pop-api/integration-tests/contracts/dao/lib.rs)
- Demo: [https://www.youtube.com/watch?v=Ll3nlshPg_U](https://www.youtube.com/watch?v=Ll3nlshPg_U)
</aside>

## 1. Introduction

The `Dao` smart contract uses:

- **NFT Registration and Verification**: Users can register NFTs from a specified parachain, verify their ownership, and mint equivalent NFTs on the local chain.
- **Cross-Chain Transactions**: The contract allows execution of transactions on the target parachain using XCM.
- **Register Callbacks:** The messaging API allows registering callbacks for automatically returning the response of the message.
- **Event-Driven Feedback**: The contract emits events to notify users of key actions and their outcomes.

This contract leverages the `pop_api` crate for runtime interactions.

---

## 2. Setup

### Deployment

To deploy the `Dao` contract, you need to:

- **Provide Sufficient Endowment**: Ensure the contract is deployed with enough funds, as the constructor transfers a portion of the endowment to the contract’s account on the destination parachain.
- **Call the Constructor**: The `new()` constructor initializes the contract and performs the following actions:
    - Initializes an `NftVerifier` instance with a hardcoded parachain ID (`1000`) and collection ID (`0`) for verifying NFTs.
    - Creates a new NFT collection on the local chain using the `nonfungibles` API.
    - Transfers 10% of the endowment (via `env().transferred_value() / 10`) as a reserve asset to the contract’s account on the destination parachain (parachain `1000`) using XCM. The asset transferred is the native token of the parent chain (`Location::parent()`).

### Configuration

- **Parachain and Collection IDs**: The `NftVerifier` uses a hardcoded parachain ID of `1000` and collection ID of `0`. If your use case requires different values, modify the `NftVerifier::new(1000, 0)` call in the constructor.
- **Asset Specification**: The reserve transfer assumes the parent chain’s native token. Adjust the `asset` definition in the constructor if you need to transfer a different asset.

---

## 3. Registering an NFT

### Function: `register(height: u32, item: ItemId)`

- **Purpose**: Initiates the verification of an NFT’s ownership on the target parachain (ID `1000`).
- **Parameters**:
    - `height: u32`: The block height at which to query the NFT ownership on the other parachain.
    - `item: ItemId`: The ID of the NFT to register (a `u32` value).
- **Process**:
    - The caller’s `AccountId` is retrieved via `self.env().caller()`.
    - The `NftVerifier::verify()` function generates a storage key and sends an ISMP `get` request to query the NFT ownership on the target parachain.
    - The registration status for the `item` is set to `Pending` in the `registered_items` mapping.
- **Events**: Emits a `RegistrationRequested` event with the caller’s `AccountId` and the `ItemId`.

### Usage Example

To register an NFT with ID `42` at block height `1000`:

- Call `register(1000, 42)` from your account.

---

## 4. Completing Registration

### Callback: `complete_registration(id: MessageId, values: Vec<StorageValue>)`

- **Selector**: `0x57ad942b`
- **Purpose**: Processes the asynchronous response from the NFT verification request.
- **Parameters**:
    - `id: MessageId`: The ID of the verification request (a `u64` value).
    - `values: Vec<StorageValue>`: The storage values returned from the ISMP `get` request.
- **Process**:
    - Retrieves the `(AccountId, ItemId)` pair associated with the `MessageId` from the `requests` mapping.
    - Checks if the NFT is owned by the user (i.e., `values[0].value.is_some()`):
        - If verified, mints a new NFT in the local collection with the next available `ItemId` (`next_item_id`), increments `next_item_id`, and updates the `registered_items` status to `Used`.
        - If not verified, no new NFT is minted.
    - Emits a `RegistrationCompleted` event with the result.
- **Events**:
    - `RegistrationCompleted { account: AccountId, verified_item: ItemId, membership: Option<ItemId> }`, where `membership` is `Some(ItemId)` if an NFT was minted, or `None` if verification failed.

### Outcome

- Success: A new NFT is minted, and you receive a membership token.
- Failure: No NFT is minted (e.g., if the NFT isn’t owned by the caller at the specified height).

---

## 5. Executing Transactions

### Function: `transact(call: Vec<u8>)`

- **Purpose**: Sends an XCM to execute a transaction on the target parachain (ID `1000`).
- **Parameters**:
    - `call: Vec<u8>`: The encoded call data to execute on the target parachain.
- **Process**:
    - Constructs an XCM message that:
        - Withdraws fees from the parent chain’s native token.
        - Buys execution on the target parachain.
        - Executes the provided `call` with a weight of `500,000,000` ref-time and `500,000` proof-size.
        - Sets up a query to receive the transaction’s result, with a callback to `process_transfer_result`.
    - Sends the XCM message and records the request in `next_request`.
- **Events**: Emits an `XcmRequested` event with the request `id`, `query_id`, and XCM `hash`.

### Usage Example

To execute a call on parachain `1000`:

- Prepare the encoded call data (e.g., `0x1234...`) and call `transact(call_data)`.

---

## 6. Processing Transfer Results

### Callback: `process_transfer_result(id: MessageId, response: Response)`

- **Selector**: `0x641b0b03`
- **Purpose**: Handles the response from the XCM transaction executed via `transact`.
- **Parameters**:
    - `id: MessageId`: The ID of the transaction request.
    - `response: Response`: The response from the target parachain.
- **Process**:
    - If the response indicates success (`Response::DispatchResult(MaybeErrorCode::Success)`), emits a `TransferCompleted` event.
    - Other responses (e.g., errors) are currently ignored but could be extended for additional handling.
- **Events**: Emits `TransferCompleted` on success.

---

## 7. Events

The contract emits the following events to provide feedback:

- **`RegistrationRequested`**
    - **Fields**: `{ account: AccountId, item: ItemId }`
    - **When**: Emitted when an NFT registration is requested.
- **`RegistrationCompleted`**
    - **Fields**: `{ account: AccountId, verified_item: ItemId, membership: Option<ItemId> }`
    - **When**: Emitted when the verification process completes, indicating whether a membership NFT was minted.
- **`TransferCompleted`**
    - **Fields**: None
    - **When**: Emitted when an XCM transaction succeeds.
- **`XcmRequested`**
    - **Fields**: `{ id: MessageId, query_id: QueryId, hash: XcmHash }`
    - **When**: Emitted when an XCM message is sent, providing tracking details.

---

## 8. Helper Functions

These internal functions support the contract’s operations:

- **`create_collection(owner: AccountId)`**: Creates a new NFT collection with transferable items disabled and issuer-only minting.
- **`generate_key(account: AccountId, collection_id: u32, item_id: u32)`**: Generates a storage key for querying NFT ownership in the `Account` storage map on the target parachain.
- **`blake2_128_concat(input: &[u8])`**: Hashes input using `Blake2x128` and concatenates it with the original input.
- **`hashed_account(para_id: u32, account_id: AccountId)`**: Computes the account’s representation on another parachain.

---

## Additional Considerations

- **Parachain and Collection IDs**: Hardcoded to `1000` and `0`. Modify the code if your setup differs.
- **Storage Key Generation**: The `generate_key` function assumes a specific storage layout (e.g., the `Account` map in the NFTs pallet). Verify compatibility with your target parachain.
- **Asset Transfers**: The constructor uses the parent chain’s native token. Adjust if using a different asset.
- **Weights and Fees**: Fixed weights (e.g., `500,000,000` for `transact`) and fees (e.g., 1% of balance) may need tuning based on actual costs.
- **Error Handling**: The contract defines an `Error` enum (e.g., `StatusCode`, `NotReady`). Handle these errors in your application logic.

---
