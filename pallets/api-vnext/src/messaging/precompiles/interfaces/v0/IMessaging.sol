// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.30;

/**
 * @title The messaging precompile offers a general interface for cross-chain messaging operations.
 */
interface IMessaging {
    /**
     * @notice Returns the response to a message (if any).
     * @dev A non-existent message identifier will return an empty response, which could also be a valid response depending on the source message.
     * @param message The message identifier.
     * @return response The response to a message.
     */
    function getResponse(
        uint64 message
    ) external returns (bytes memory response);

    /**
     * @notice Polls the status of a message.
     * @param message The message identifier to poll.
     * @return status The status of the message.
     */
    function pollStatus(uint64 message) external returns (MessageStatus status);

    /**
     * @notice Remove a completed or timed-out message.
     * @dev Allows users to clean up storage and reclaim deposits for messages that have concluded.
     * @param message The identifier of the message to remove.
     */
    function remove(uint64 message) external;

    /**
     * @notice Remove a batch of completed or timed-out messages.
     * @dev Allows users to clean up storage and reclaim deposits for messages that have concluded.
     * @param messages A set of identifiers of messages to remove (bounded by `MaxRemovals`).
     */
    function remove(uint64[] calldata messages) external;

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
}
