// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.30;

/**
 * @title The fungibles precompile offers a streamlined interface for interacting with fungible
 * tokens. The goal is to provide a simplified, consistent API that adheres to standards in the
 * smart contract space.
 */
interface IFungibles {
    /**
     * @notice Transfers `value` amount of tokens from the caller's account to account `to`.
     * @param token The token to transfer.
     * @param to The recipient account.
     * @param value The number of tokens to transfer.
     */
    function transfer(uint32 token, address to, uint256 value) external;

    /**
     * @notice Transfers `value` amount tokens on behalf of `from` to account `to`.
     * @param token The token to transfer.
     * @param from The account from which the token balance will be withdrawn.
     * @param to The recipient account.
     * @param value The number of tokens to transfer.
     */
    function transferFrom(
        uint32 token,
        address from,
        address to,
        uint256 value
    ) external;

    /**
     * @notice Approves `spender` to spend `value` amount of tokens on behalf of the caller.
     * @param token The token to approve.
     * @param spender The account that is allowed to spend the tokens.
     * @param value The number of tokens to approve.
     */
    function approve(uint32 token, address spender, uint256 value) external;

    /**
     * @notice Increases the allowance of `spender` by `value` amount of tokens.
     * @param token The token to have an allowance increased.
     * @param spender The account that is allowed to spend the tokens.
     * @param value The number of tokens to increase the allowance by.
     * @return allowance The resulting allowance of `spender`.
     */
    function increaseAllowance(
        uint32 token,
        address spender,
        uint256 value
    ) external returns (uint256 allowance);

    /**
     * @notice Decreases the allowance of `spender` by `value` amount of tokens.
     * @param token The token to have an allowance decreased.
     * @param spender The account that is allowed to spend the tokens.
     * @param value The number of tokens to decrease the allowance by.
     * @return allowance The resulting allowance of `spender`.
     */
    function decreaseAllowance(
        uint32 token,
        address spender,
        uint256 value
    ) external returns (uint256 allowance);

    /**
     * @notice Create a new token with an automatically generated identifier.
     * @param admin The account that will administer the token.
     * @param minBalance The minimum balance required for accounts holding this token.
     * @return id The resulting identifier of the token.
     */
    function create(
        address admin,
        uint256 minBalance
    ) external returns (uint32 id);

    /**
     * @notice Start the process of destroying a token.
     * @dev See `pallet-assets` documentation for more information. Related dispatchables are`destroy_accounts`, `destroy_approvals`, `finish_destroy`.
     * @param token The token to be destroyed.
     */
    function startDestroy(uint32 token) external;

    /**
     * @notice Set the metadata for a token.
     * @param token The token to update.
     * @param name The user friendly name of this token.
     * @param symbol The exchange symbol for this token.
     * @param decimals The number of decimals this token uses to represent one unit.
     */
    function setMetadata(
        uint32 token,
        string calldata name,
        string calldata symbol,
        uint8 decimals
    ) external;

    /**
     * @notice Clear the metadata for a token.
     * @param token The token to update.
     */
    function clearMetadata(uint32 token) external;

    /**
     * @notice Creates `value` amount of tokens and assigns them to `account`, increasing the total supply.
     * @param token The token to mint.
     * @param token The account to be credited with the created tokens.
     * @param value The number of tokens to mint.
     */
    function mint(uint32 token, address account, uint256 value) external;

    /**
     * @notice Destroys `value` amount of tokens from `account`, reducing the total supply.
     * @param token The token to burn.
     * @param account The account from which the tokens will be destroyed.
     * @param value The number of tokens to destroy.
     */
    function burn(uint32 token, address account, uint256 value) external;

    /**
     * @notice Total token supply for a specified token.
     * @param token The token.
     */
    function totalSupply(uint32 token) external view returns (uint256);

    /**
     * @notice Account balance for a specified `token` and `owner`.
     * @param token The token.
     * @param owner The owner of the token.
     */
    function balanceOf(
        uint32 token,
        address owner
    ) external view returns (uint256);

    /**
     * @notice Allowance for a `spender` approved by an `owner`, for a specified `token`.
     * @param token The token.
     * @param owner The owner of the token.
     * @param spender The spender with an allowance.
     */
    function allowance(
        uint32 token,
        address owner,
        address spender
    ) external view returns (uint256);

    /**
     * @notice Name of the specified token.
     */
    function name(uint32 token) external view returns (string memory);

    /**
     * @notice Symbol for the specified token.
     */
    function symbol(uint32 token) external view returns (string memory);

    /**
     * @notice Decimals for the specified token.
     */
    function decimals(uint32 token) external view returns (uint8);

    /**
     * @notice Whether the specified token exists.
     */
    function exists(uint32 token) external view returns (bool);

    /**
     * @notice Event emitted when allowance by `owner` to `spender` changes.
     * @param token The token.
     * @param owner The owner providing the allowance.
     * @param spender The beneficiary of the allowance.
     * @param value The new allowance amount.
     */
    event Approval(uint32 token, address owner, address spender, uint256 value);

    /**
     * @notice Event emitted when a token transfer occurs.
     * @param token The token.
     * @param from The source of the transfer. The zero address when minting.
     * @param to The recipient of the transfer. The zero address when burning.
     * @param value The amount transferred (or minted/burned).
     */
    event Transfer(
        uint32 indexed token,
        address indexed from,
        address indexed to,
        uint256 value
    );

    /**
     * @notice Event emitted when a token is created.
     * @param id The token identifier.
     * @param creator The creator of the token.
     * @param admin The administrator of the token.
     */
    event Created(uint32 id, address creator, address admin);

    /// @notice The metadata provided is invalid.
    error BadMetadata();
    /// @notice The account balance is insufficient.
    error InsufficientBalance();
    /// @notice The token recipient is invalid.
    error InvalidRecipient(address);
    /// @notice The minimum balance should be non-zero.
    error MinBalanceZero();
    /// @notice The signing account has no permission to do the operation.
    error NoPermission();
    /// @notice The token is not live, and likely being destroyed..
    error NotLive();
    /// @notice No approval exists that would allow the transfer.
    error Unapproved();
    /// @notice The given token identifier is unknown.
    error Unknown();
    /// @notice The `admin` address cannot be the zero address.
    error ZeroAdminAddress();
    /// @notice The recipient cannot be the zero address.
    error ZeroRecipientAddress();
    /// @notice The sender cannot be the zero address.
    error ZeroSenderAddress();
    /// @notice The specified `value` cannot be zero.
    error ZeroValue();
}

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
	DecodeError,
}
