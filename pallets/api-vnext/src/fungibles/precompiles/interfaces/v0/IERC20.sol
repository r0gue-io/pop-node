// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

/**
 * @title Interface of the ERC-20 standard as defined in the ERC.
 * Based on https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/IERC20.sol
 */
interface IERC20 {
    /**
     * @notice Emitted when `value` tokens are moved from one account (`from`) to another (`to`).
     *
     * Note that `value` may be zero.
     */
    event Transfer(address indexed from, address indexed to, uint256 value);
    /**
     * @notice Emitted when the allowance of a `spender` for an `owner` is set by a call to {approve}.
     * `value` is the new allowance.
     */
    event Approval(
        address indexed owner,
        address indexed spender,
        uint256 value
    );

    /**
     * @notice Returns the value of tokens in existence.
     */
    function totalSupply() external view returns (uint256);

    /**
     * @notice Returns the value of tokens owned by `account`.
     */
    function balanceOf(address account) external view returns (uint256);

    /**
     * @notice Moves a `value` amount of tokens from the caller's account to `to`.
     *
     * Returns a boolean value indicating whether the operation succeeded.
     *
     * Emits a {Transfer} event.
     */
    function transfer(address to, uint256 value) external returns (bool);

    /**
     * @notice Returns the remaining number of tokens that `spender` will be allowed to spend on
     * behalf of `owner` through {transferFrom}. This is zero by default.
     *
     * This value changes when {approve} or {transferFrom} are called.
     */
    function allowance(
        address owner,
        address spender
    ) external view returns (uint256);

    /**
     * @notice Sets a `value` amount of tokens as the allowance of `spender` over the caller's
     * tokens.
     *
     * Returns a boolean value indicating whether the operation succeeded.
     *
     * IMPORTANT: Beware that changing an allowance with this method brings the risk that
     * someone may use both the old and the new allowance by unfortunate transaction ordering.
     * One possible solution to mitigate this race condition is to first reduce the spender's
     * allowance to 0 and set the desired value afterwards:
     * https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729
     *
     * Emits an {Approval} event.
     */
    function approve(address spender, uint256 value) external returns (bool);

    /**
     * @notice Moves a `value` amount of tokens from `from` to `to` using the allowance mechanism.
     * `value` is then deducted from the caller's allowance.
     *
     * Returns a boolean value indicating whether the operation succeeded.
     *
     * Emits a {Transfer} event.
     */
    function transferFrom(
        address from,
        address to,
        uint256 value
    ) external returns (bool);

    // Extensions: `sol!` macro does not support inheritance, so extensions need to be included in same interface

    // IERC20Metadata: Interface for the optional metadata functions from the ERC-20 standard.
    // Source: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/extensions/IERC20Metadata.sol

    /**
     * @notice Returns the name of the token.
     */
    function name() external view returns (string memory);

    /**
     * @notice Returns the symbol of the token.
     */
    function symbol() external view returns (string memory);

    /**
     * @notice Returns the decimals places of the token.
     */
    function decimals() external view returns (uint8);

    /**
     * @notice Indicates a failure with the `spender`’s `allowance`. Used in transfers.
     * @param spender Address that may be allowed to operate on tokens without being their owner.
     * @param allowance Amount of tokens a `spender` is allowed to operate with.
     * @param needed Minimum amount required to perform a transfer.
     */
    error ERC20InsufficientAllowance(
        address spender,
        uint256 allowance,
        uint256 needed
    );
    /**
     * @notice Indicates an error related to the current `balance` of a `sender`. Used in transfers.
     * @param sender Address whose tokens are being transferred.
     * @param balance Current balance for the interacting account.
     * @param needed Minimum amount required to perform a transfer.
     */
    error ERC20InsufficientBalance(
        address sender,
        uint256 balance,
        uint256 needed
    );
    /// @notice Indicates an error related to a specified `value`.
    error ERC20InsufficientValue();
    /**
     * @notice Indicates a failure with the token `receiver`. Used in transfers.
     * @param receiver Address to which tokens are being transferred.
     */
    error ERC20InvalidReceiver(address receiver);
    /**
     * @notice Indicates a failure with the token `sender`. Used in transfers.
     * @param sender Address whose tokens are being transferred.
     */
    error ERC20InvalidSender(address sender);
    /**
     * @notice Indicates a failure with the `spender` to be approved. Used in approvals.
     * @param spender Address that may be allowed to operate on tokens without being their owner.
     */
    error ERC20InvalidSpender(address spender);
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
