// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.30;

/**
 * @title The ISMP precompile offers a streamlined interface for messaging using the Interoperable State Machine Protocol.
 */
interface IISMP {
    /**
     * @notice Submit a new ISMP `Get` request.
     * @dev Sends a `Get` request through ISMP.
     * @param request The ISMP `Get` message containing query details.
     * @param fee The fee to be paid to relayers.
     * @return id A unique message identifier.
     */
    function get(
        Get calldata request,
        uint256 fee
    ) external returns (uint64 id);

    /**
     * @notice Submit a new ISMP `Get` request.
     * @dev Sends a `Get` request through ISMP with a callback to handle the response.
     * @param fee The fee to be paid to relayers.
     * @param callback The callback to execute upon receiving a response.
     * @return id A unique message identifier.
     */
    function get(
        Get calldata request,
        uint256 fee,
        Callback calldata callback
    ) external returns (uint64 id);

    /**
     * @notice Returns the response to a message (if any).
     * @dev A non-existent message identifier will return an empty response, which could also be a valid response depending on the source message.
     * @param message The message identifier.
     * @return response The response to a message.
     */
    function getResponse(uint64 message) external returns (bytes memory response);

    /**
     * @notice The identifier of this chain.
     * @return id The identifier of this chain.
     */
    function id() external returns (uint32 id);

    /**
     * @notice Polls the status of a message.
     * @param message The message identifier to poll.
     * @return status The status of the message.
     */
    function pollStatus(uint64 message) external returns (MessageStatus status);

    /**
     * @notice Submit a new ISMP `Post` request.
     * @dev Sends a `Post` message through ISMP with arbitrary data.
     * @param request The ISMP `Post` message containing the payload.
     * @param fee The fee to be paid to relayers.
     * @return id A unique message identifier.
     */
    function post(
        Post calldata request,
        uint256 fee
    ) external returns (uint64 id);

    /**
     * @notice Submit a new ISMP `Post` request.
     * @dev Sends a `Post` message through ISMP with arbitrary data and a callback.
     * @param request The ISMP `Post` message containing the payload.
     * @param fee The fee to be paid to relayers.
     * @param callback The callback to execute upon receiving a response.
     * @return id A unique message identifier.
     */
    function post(
        Post calldata request,
        uint256 fee,
        Callback calldata callback
    ) external returns (uint64 id);

    /**
     * @notice Remove a completed or timed-out message.
     * @dev Allows users to clean up storage and reclaim deposits for messages that have concluded.
     * @param message The message identifier to remove.
     */
    function remove(uint64 message) external;

    /**
     * @notice Remove a batch of completed or timed-out messages.
     * @dev Allows users to clean up storage and reclaim deposits for messages that have concluded.
     * @param messages A set of message identifiers to remove (bounded by `MaxRemovals`).
     */
    function remove(uint64[] calldata messages) external;

    /// @notice A GET request, intended to be used for sending outgoing requests
    struct Get {
        /// @custom:property The destination state machine of this request.
        uint32 destination;
        /// @custom:property Height at which to read the state machine.
        uint64 height;
        /// @custom:property Relative from the current timestamp at which this request expires in seconds.
        uint64 timeout;
        /// @custom:property Some application-specific metadata relating to this request.
        bytes context;
        /// @custom:property Raw Storage keys that would be used to fetch the values from the counterparty.
        bytes[] keys;
    }

    /// @notice A POST request, intended to be used for sending outgoing requests.
    struct Post {
        /// @custom:property The destination state machine of this request.
        uint32 destination;
        /// @custom:property Relative from the current timestamp at which this request expires in seconds.
        uint64 timeout;
        /// @custom:property Encoded request data.
        bytes data;
    }

    /// @notice A verified storage value.
    struct StorageValue{
        /// @custom:property The request storage key.
        bytes key;
        /// @custom:property The verified value.
        Value value;
    }

    /// @notice A verified storage value.
    struct Value {
        /// @custom:property Whether a value exists.
        bool exists;
        /// @custom:property The verified value.
        bytes value;
    }

    /**
     * @notice A GET has been dispatched via ISMP.
     * @param origin The origin of the request.
     * @param id The identifier of the message.
     * @param commitment The ISMP request commitment.
     */
    event GetDispatched(address origin, uint64 id, bytes32 commitment);

    /**
     * @notice A GET has been dispatched via ISMP.
     * @param origin The origin of the request.
     * @param id The identifier of the message.
     * @param commitment The ISMP request commitment.
     * @param callback The callback to be used to return the response.
     */
    event GetDispatched(address origin, uint64 id, bytes32 commitment, Callback callback);

    /**
     * @notice A POST has been dispatched via ISMP.
     * @param origin The origin of the request.
     * @param id The identifier of the message.
     * @param commitment The ISMP request commitment.
     */
    event PostDispatched(address origin, uint64 id, bytes32 commitment);

    /**
     * @notice A POST has been dispatched via ISMP.
     * @param origin The origin of the request.
     * @param id The identifier of the message.
     * @param commitment The ISMP request commitment.
     * @param callback The callback to be used to return the response.
     */
    event PostDispatched(address origin, uint64 id, bytes32 commitment, Callback callback);

    /// @dev The context exceeds the maximum allowed size.
    error MaxContextExceeded();
    /// @dev The data exceeds the maximum allowed size.
    error MaxDataExceeded();
    /// @dev A key exceeds the maximum allowed size.
    error MaxKeyExceeded();
    /// @dev The number of keys exceeds the maximum allowed size.
    error MaxKeysExceeded();
}

/**
 * @title A callback for handling responses to ISMP `Get` requests.
 */
interface IGetResponse {
    /**
     * @notice Handles a response to an ISMP `Get` request.
     * @param id The identifier of the originating message.
     * @param response The values derived from the state proof.
     */
    function onGetResponse(uint64 id, IISMP.StorageValue[] memory response) external;
}

/**
 * @title A callback for handling responses to ISMP `Post` requests.
 */
interface IPostResponse {
    /**
     * @notice Handles a response to an ISMP `Post` request.
     * @param id The identifier of the originating message.
     * @param response The response message.
     */
    function onPostResponse(uint64 id, bytes memory response) external;
}

/// @notice A message callback.
struct Callback {
    /// @custom:property The contract address to which the callback should be sent.
    address destination;
    /// @custom:property The encoding used for the data going to the contract.
    Encoding encoding;
    /// @custom:property The message selector to be used for the callback.
    bytes4 selector;
    /// @custom:property The pre-paid weight used as a gas limit for the callback.
    Weight gasLimit;
    /// @custom:property The storage deposit limit for the callback.
    uint256 storageDepositLimit;
}

/**
 * @notice The specificiation of how data must be encoded before being sent to a contract.
 */
enum Encoding {
    Scale,
    SolidityAbi
}

/**
 * @notice The status of a message.
 */
enum MessageStatus {
    NotFound,
    Pending,
    Complete,
    Timeout
}

/**
 * @notice One or more messages have been removed for the account.
 * @param account The origin of the messages.
 * @param messages The messages which were removed.
 */
event Removed(address account, uint64[] messages);

/// @notice The weight of/for a transaction.
struct Weight {
    /// @custom:property The weight of computational time used based on some reference hardware.
    uint64 refTime;
    /// @custom:property The weight of storage space used by proof of validity.
    uint64 proofSize;
}

/// @dev The specified encoding is invalid.
error InvalidEncoding();
/// @dev The message was not found.
error MessageNotFound();
/// @dev The request is pending.
error RequestPending();
/// @dev The number of messages exceeds the limit.
error TooManyMessages();
