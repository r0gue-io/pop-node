/// @title INonfungibles - Interface for interacting with the pallet_nfts logic
/**
 * @dev The nonfungibles precompile offers a streamlined interface for interacting with a nonfungible token. The goal is to
 * provide a simplified, consistent API that adheres to standards in the smart contract space.
 */
// TODO: consider size of id -> uint256 (word)
interface INonfungibles {
    /**
     * @dev Approves `operator` to transfer all items in `collection` on behalf of the caller until `deadline`.
     */
    function approve(
        uint32 collection,
        address operator,
        bool approved,
        uint32 deadline
    ) external;

    /**
     * @dev Approves `operator` to transfer the given `item` on behalf of the caller until `deadline`.
     */
    function approve(
        uint32 collection,
        address operator,
        uint32 item,
        bool approved,
        uint32 deadline
    ) external;

    /**
     * @dev Transfers the specified `item` in `collection` from caller to `to`.
     */
    function transfer(
        uint32 collection,
        address to,
        uint32 item
    ) external;

    /**
     * @dev Creates a new NFT collection with the given `admin` and `config`.
     */
    function create(
        address admin,
        bytes calldata config
    ) external returns (uint32 collection);

    /**
     * @dev Destroys the specified `collection` with the given `witness`.
     */
    function destroy(
        uint32 collection,
        bytes calldata witness
    ) external;

    /**
     * @dev Sets an attribute on entire `collection` under the given `namespace` and `key`.
     */
    function setAttribute(
        uint32 collection,
        bytes calldata namespace,
        bytes calldata key,
        bytes calldata value
    ) external;

    /**
     * @dev Sets an attribute on `item` in a `collection` under the given `namespace` and `key`.
     */
    function setAttribute(
        uint32 collection,
        uint32 item,
        bytes calldata namespace,
        bytes calldata key,
        bytes calldata value
    ) external;

    /**
     * @dev Clears an attribute from an entire `collection` under the given `namespace` and `key`.
     */
    function clearAttribute(
        uint32 collection,
        bytes calldata namespace,
        bytes calldata key
    ) external;

    /**
     * @dev Clears an attribute from `item` in a `collection` under the given `namespace` and `key`.
     */
    function clearAttribute(
        uint32 collection,
        uint32 item,
        bytes calldata namespace,
        bytes calldata key
    ) external;

    /**
     * @dev Sets metadata for the specified `item` in `collection`.
     */
    function setMetadata(
        uint32 collection,
        uint32 item,
        bytes calldata data
    ) external;

    /**
     * @dev Sets metadata for the specified `collection`.
     */
    function setMetadata(
        uint32 collection,
        bytes calldata data
    ) external;

    /**
     * @dev Clears metadata for the specified `collection`.
     */
    function clearMetadata(
        uint32 collection
    ) external;

    /**
     * @dev Clears metadata for the specified `item` in `collection`.
     */
    function clearMetadata(
        uint32 collection,
        uint32 item
    ) external;

    /**
     * @dev Sets the max supply for the specified `collection`.
     */
    function setMaxSupply(
        uint32 collection,
        uint32 maxSupply
    ) external;

    /**
     * @dev Approves `delegate` to manage attributes of `item` in `collection`.
     */
    function approveItemAttributes(
        uint32 collection,
        uint32 item,
        address delegate
    ) external;

    /**
     * @dev Cancels attribute approval for `delegate` on `item` in `collection` using the given `witness`.
     */
    function cancelItemAttributesApproval(
        uint32 collection,
        uint32 item,
        address delegate,
        bytes calldata witness
    ) external;

    /**
     * @dev Clears all transfer approvals from the specified `item` in `collection`.
     */
    function clearAllApprovals(
        uint32 collection,
        uint32 item
    ) external;

    /**
     * @dev Clears collection-level approvals, with a `limit` to how many to remove.
     */
    function clearCollectionApprovals(
        uint32 collection,
        uint32 limit
    ) external;

    /**
     * @dev Mints a new `item` in `collection` to `to` with optional `witness`.
     */
    function mint(
        uint32 collection,
        address to,
        uint32 item,
        bytes calldata witness
    ) external;

    /**
     * @dev Burns the specified `item` in `collection`.
     */
    function burn(
        uint32 collection,
        uint32 item
    ) external;

    /**
     * @dev Returns the balance (number of owned items) for `owner` in `collection`.
     */
    function balanceOf(
        uint32 collection,
        address owner
    ) external view returns (uint32);

    /**
     * @dev Returns the owner of the specified `item` in `collection`.
     */
    function ownerOf(
        uint32 collection,
        uint32 item
    ) external view returns (address);

    /**
     * @dev Returns whether `operator` is approved to transfer all items on behalf of `owner`.
     */
    function allowance(
        uint32 collection,
        address owner,
        address operator
    ) external view returns (bool);

    /**
     * @dev Returns if `operator` is approved to transfer the `item` on behalf of `owner`.
     */
    function allowance(
        uint32 collection,
        address owner,
        address operator,
        uint32 item
    ) external view returns (bool);

    /**
     * @dev Returns the total number of items minted in `collection`.
     */
    function totalSupply(
        uint32 collection
    ) external view returns (uint32);

    /**
     * @dev Returns the attribute value of `key` under `namespace` for `collection`.
     */
    function getAttribute(
        uint32 collection,
        bytes calldata namespace,
        bytes calldata key
    ) external view returns (string memory);

    /**
     * @dev Returns the attribute value of `key` under `namespace` for given `item` in `collection`.
     */
    function getAttribute(
        uint32 collection,
        uint32 item,
        bytes calldata namespace,
        bytes calldata key
    ) external view returns (string memory);

    /**
     * @dev Returns the metadata for the specified `item` in `collection`.
     */
    function itemMetadata(
        uint32 collection,
        uint32 item
    ) external view returns (string memory);

    /**
     * @dev Emitted when a token transfer occurs.
     * `from` is the address tokens are sent from (zero address if minted).
     * `to` is the address tokens are sent to (zero address if burned).
     */
    event Transfer(address indexed from, address indexed to, uint32 item);

    /**
     * @dev Emitted when an approval is set or revoked for an item.
     */
    event ItemApproval(address indexed owner, address indexed operator, uint32 item, bool approved);

    /**
     * @dev Emitted when an approval is set or revoked for a collection.
     */
    event CollectionApproval(address indexed owner, address indexed operator, uint32 collection, bool approved);

    /**
     * @dev Emitted when an attribute is set on an item.
     */
    event ItemAttributeSet(uint32 indexed item, bytes key, bytes data);

    /**
     * @dev Emitted when an attribute is set on a collection.
     */
    event CollectionAttributeSet(uint32 indexed collection, bytes key, bytes data);
}
