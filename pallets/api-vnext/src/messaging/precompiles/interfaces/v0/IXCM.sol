// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.30;

/**
 * @title The XCM precompile offers a streamlined interface for messaging using Polkadot's Cross-Consensus Messaging (XCM).
 * @dev An extension of https://github.com/paritytech/polkadot-sdk/blob/master/polkadot/xcm/pallet-xcm/src/precompiles/IXcm.sol
 */
interface IXCM {

    /**
     * @notice The current block number.
     */
    function blockNumber() external view returns (uint32 result);

    /**
     * @notice Execute an XCM message from a local, signed, origin.
     * @param message A SCALE-encoded versioned XCM message.
     * @param weight The maximum allowed weight for execution.
     * @return result A SCALE-encoded dispatch result.
     */
    function execute(
        bytes calldata message,
        Weight calldata weight
    ) external returns (bytes memory result);

    /**
     * @notice Returns the response to a message (if any).
     * @dev A non-existent message identifier will return an empty response, which could also be a valid response depending on the source message.
     * @param message The message identifier.
     * @return response The response to a message.
     */
    function getResponse(uint64 message) external returns (bytes memory response);

    /**
     * @notice Initiate a new XCM query.
     * @dev Starts a query using the XCM interface, specifying a responder and timeout block.
     * @param responder A SCALE-encoded versioned location of the XCM responder.
     * @param timeout Block number after which the query should timeout. A future block number is required.
     * @return id A unique message identifier.
     */
    function newQuery(
        bytes calldata responder,
        uint32 timeout
    ) external returns (uint64 id);

    /**
     * @notice Initiate a new XCM query.
     * @dev Starts a query using the XCM interface, specifying a responder and timeout block.
     * @param responder A SCALE-encoded versioned location of the XCM responder.
     * @param timeout Block number after which the query should timeout. A future block number is required.
     * @param callback The callback to execute upon receiving a response.
     * @return id A unique message identifier.
     */
    function newQuery(
        bytes calldata responder,
        uint32 timeout,
        Callback calldata callback
    ) external returns (uint64 id);

    /**
     * @notice Polls the status of a message.
     * @param message The message identifier to poll.
     * @return status The status of the message.
     */
    function pollStatus(uint64 message) external returns (MessageStatus status);

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

    /**
     * @notice Send an XCM from a given origin.
     * @param destination The SCALE-encoded versioned location for the destination of the message.
     * @param message A SCALE-encoded versioned XCM message.
     * @return result A SCALE-encoded dispatch result.
     */
    function send(
        bytes calldata destination,
        bytes calldata message
    ) external returns (bytes memory result);

    /**
     * @notice A XCM query has been created.
     * @param account The origin of the request.
     * @param id The identifier of the message.
     * @param queryId The identifier of the created XCM query.
     */
    event QueryCreated(address account, uint64 id, uint64 queryId);

    /**
     * @notice A XCM query has been created.
     * @param account The origin of the request.
     * @param id The identifier of the message.
     * @param queryId The identifier of the created XCM query.
     * @param callback The callback to be used to return the response.
     */
    event QueryCreated(address account, uint64 id, uint64 queryId, Callback callback);
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
    Weight weight;
}

/**
 * @notice The specificiation of how data must be encoded before being sent to a contract.
 */
enum Encoding {
    Scale,
    SolidityAbi
}

/**
 * @notice One or more messages have been removed for the account.
 * @param account The origin of the messages.
 * @param messages The messages which were removed.
 */
event Removed(address account, uint64[] messages);

/**
 * @notice The status of a message.
 */
enum MessageStatus {
    NotFound,
    Pending,
    Complete,
    Timeout
}

/// @notice The weight of/for a transaction.
struct Weight {
    /// @custom:property The weight of computational time used based on some reference hardware.
    uint64 refTime;
    /// @custom:property The weight of storage space used by proof of validity.
    uint64 proofSize;
}
