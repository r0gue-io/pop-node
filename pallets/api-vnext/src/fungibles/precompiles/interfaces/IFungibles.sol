// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.30;

/**
 * @title The fungibles precompile offers a streamlined interface for interacting with fungible
 * tokens. The goal is to provide a simplified, consistent API that adheres to standards in the
 * smart contract space.
 */
// TODO: consider size of id -> uint256 (word)
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
}
