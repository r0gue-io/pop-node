/**
 * @dev The fungibles precompile offers a streamlined interface for interacting with fungible tokens. The goal is to
 * provide a simplified, consistent API that adheres to standards in the smart contract space.
 */
// TODO: consider size of id -> uint256 (word)
interface IFungibles {
    /**
     * @dev Create a new token with an automatically generated identifier.
     */
    function create(
        address admin,
        uint256 minBalance
    ) external returns (uint32 id);

    /**
     * @dev Set the metadata for a token.
     */
    function setMetadata(
        uint32 id,
        string calldata name,
        string calldata symbol,
        uint8 decimals
    ) external;

    /**
     * @dev Clear the metadata for a token.
     */
    function clearMetadata(uint32 id) external;

    /**
     * @dev Creates `value` amount of tokens and assigns them to `account`, increasing the total supply.
     */
    function mint(uint32 id, address account, uint256 value) external;

    /**
     * @dev Transfers `value` amount of tokens from the caller's account to account `to`.
     */
    function transfer(uint32 id, address to, uint256 value) external;

    /**
     * @dev Approves `spender` to spend `value` amount of tokens on behalf of the caller.
     */
    function approve(uint32 id, address spender, uint256 value) external;

    /**
     * @dev Transfers `value` amount tokens on behalf of `from` to account `to`.
     */
    function transferFrom(
        uint32 id,
        address from,
        address to,
        uint256 value
    ) external;

    /**
     * @dev Destroys `value` amount of tokens from `account`, reducing the total supply.
     */
    function burn(uint32 id, address account, uint256 value) external;

    /**
     * @dev Start the process of destroying a token.
     */
    function startDestroy(uint32 id) external;

    /**
     * @dev Whether a specified token exists.
     */
    function exists(uint32 id) external view returns (bool);

    /**
     * @dev Event emitted when allowance by `owner` to `spender` changes.
     */
    event Approval(uint32 id, address owner, address spender, uint256 value);
    /**
     * @dev Event emitted when a token is created.
     */
    event Created(uint32 id, address creator, address admin);
    /**
     * @dev Event emitted when a token transfer occurs.
     */
    event Transfer(
        uint32 indexed id,
        address indexed from,
        address indexed to,
        uint256 value
    );
}
