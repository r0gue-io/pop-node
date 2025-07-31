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
     * @notice The identifier of this chain.
     * @return id The identifier of this chain.
     */
    function id() external returns (uint32 id);

    /**
     * @notice Initiate a new XCM query.
     * @dev Starts a query using the XCM interface, specifying a responder and timeout block.
     * @param responder A SCALE-encoded versioned location of the XCM responder.
     * @param timeout Block number after which the query should timeout. A future block number is required.
     * @return id A unique message identifier.
     * @return queryId The XCM query identifier.
     */
    function newQuery(
        bytes calldata responder,
        uint32 timeout
    ) external returns (uint64 id, uint64 queryId);

    /**
     * @notice Initiate a new XCM query.
     * @dev Starts a query using the XCM interface, specifying a responder and timeout block.
     * @param responder A SCALE-encoded versioned location of the XCM responder.
     * @param timeout Block number after which the query should timeout. A future block number is required.
     * @param callback The callback to execute upon receiving a response.
     * @return id A unique message identifier.
     * @return queryId The XCM query identifier.
     */
    function newQuery(
        bytes calldata responder,
        uint32 timeout,
        Callback calldata callback
    ) external returns (uint64 id, uint64 queryId);

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

    /// @dev The input failed to decode.
    error DecodingFailed();
    /// @dev Timeouts must be in the future.
    error FutureTimeoutMandatory();
    /// @dev Timeouts must be in the future.
    error FundsUnavailable();
    /// @dev Message block limit has been reached for this expiry block. Try a different timeout.
    error MaxMessageTimeoutPerBlockReached();
    /// @dev Failed to convert origin.
    error OriginConversionFailed();
}

/**
 * @title A callback for handling responses to XCM queries.
 */
interface IQueryResponse {
    /**
     * @notice Handles a response to an ISMP `Post` request.
     * @param id The identifier of the originating message.
     * @param response The response message.
     */
    function onQueryResponse(uint64 id, bytes memory response) external;
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

/// @dev The specified encoding is invalid.
error InvalidEncoding();
/// @dev The message was not found.
error MessageNotFound();
/// @dev The request is pending.
error RequestPending();
/// @dev The number of messages exceeds the limit.
error TooManyMessages();

/// @notice An arithmetic error.
error Arithmetic(ArithmeticError);
/// @title Arithmetic errors.
enum ArithmeticError {
    /// @notice Underflow.
    Underflow,
    /// @notice Overflow.
    Overflow,
    /// @notice Division by zero.
    DivisionByZero
}

/// @notice Reason why a dispatch call failed.
error Dispatch(DispatchError);
/// @title Reason why a dispatch call failed.
enum DispatchError {
    /// @notice Some error occurred.
    Other,
    /// @notice Failed to lookup some data.
    CannotLookup,
    /// @notice A bad origin.
    BadOrigin,
    /// @notice A custom error in a module.
    Module,
    /// @notice At least one consumer is remaining so the account cannot be destroyed.
    ConsumerRemaining,
    /// @notice There are no providers so the account cannot be created.
    NoProviders,
    /// @notice There are too many consumers so the account cannot be created.
    TooManyConsumers,
    /// @notice An error to do with tokens.
    Token,
    /// @notice An arithmetic error.
    Arithmetic,
    /// @notice The number of transactional layers has been reached, or we are not in a
    /// transactional layer.
    Transactional,
    /// @notice Resources exhausted, e.g. attempt to read/write data which is too large to manipulate.
    Exhausted,
    /// @notice The state is corrupt; this is generally not going to fix itself.
    Corruption,
    /// @notice Some resource (e.g. a preimage) is unavailable right now. This might fix itself later.
    Unavailable,
    /// @notice Root origin is not allowed.
    RootNotAllowed,
    /// @notice An error with tries.
    Trie
}

/**
 * @notice Reason why a pallet call failed.
 * @param index Module index, matching the metadata module index.
 * @param error Module specific error value.
 */
error Module(uint8 index, bytes4 error);

/// @notice An error to do with tokens.
error Token(TokenError);
/// @title Description of what went wrong when trying to complete an operation on a token.
enum TokenError {
    /// @notice Funds are unavailable.
    FundsUnavailable,
    /// @notice Some part of the balance gives the only provider reference to the account and thus cannot be (re)moved.
    OnlyProvider,
    /// @notice Account cannot exist with the funds that would be given.
    BelowMinimum,
    /// @notice Account cannot be created.
    CannotCreate,
    /// @notice The token in question is unknown.
    Unknown,
    /// @notice Funds exist but are frozen.
    Frozen,
    /// @notice Operation is not supported by the token.
    Unsupported,
    /// @notice Account cannot be created for a held balance.
    CannotCreateHold,
    /// @notice Withdrawal would cause unwanted loss of account.
    NotExpendable,
    /// @notice Account cannot receive the tokens.
    Blocked
}

/// @notice The number of transactional layers has been reached, or we are not in a transactional layer.
error Transactional(TransactionalError);
/// @title Errors related to transactional storage layers.
enum TransactionalError {
    /// @notice Too many transactional layers have been spawned.
    LimitReached,
    /// @notice A transactional layer was expected, but does not exist.
    NoLayer
}

/// @notice An error with tries.
error Trie(TrieError);
/// @title A runtime friendly error type for tries.
enum TrieError {
    /// @notice Attempted to create a trie with a state root not in the DB.
    InvalidStateRoot,
    /// @notice Trie item not found in the database,
    IncompleteDatabase,
    /// @notice A value was found in the trie with a nibble key that was not byte-aligned.
    ValueAtIncompleteKey,
    /// @notice Corrupt Trie item.
    DecoderError,
    /// @notice Hash is not value.
    InvalidHash,
    /// @notice The statement being verified contains multiple key-value pairs with the same key.
    DuplicateKey,
    /// @notice The proof contains at least one extraneous node.
    ExtraneousNode,
    /// @notice The proof contains at least one extraneous value which should have been omitted from the
    /// proof.
    ExtraneousValue,
    /// @notice The proof contains at least one extraneous hash reference the should have been omitted.
    ExtraneousHashReference,
    /// @notice The proof contains an invalid child reference that exceeds the hash length.
    InvalidChildReference,
    /// @notice The proof indicates that an expected value was not found in the trie.
    ValueMismatch,
    /// @notice The proof is missing trie nodes required to verify.
    IncompleteProof,
    /// @notice The root hash computed from the proof is incorrect.
    RootMismatch,
    /// @notice One of the proof nodes could not be decoded.
    DecodeError
}
