pub type FixedBytes<const N: usize> = ink::SolBytes<[u8; N]>;

/// Arithmetic errors.
#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
#[derive(Copy, Clone, ink::SolDecode, ink::SolEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
pub enum ArithmeticError {
	/// Underflow.
	Underflow,
	/// Overflow.
	Overflow,
	/// Division by zero.
	DivisionByZero,
}

/// @Reason why a dispatch call failed.
#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
#[derive(Copy, Clone, ink::SolDecode, ink::SolEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
pub enum DispatchError {
	/// Some error occurred.
	Other,
	/// Failed to lookup some data.
	CannotLookup,
	/// A bad origin.
	BadOrigin,
	/// A custom error in a module.
	Module,
	/// At least one consumer is remaining so the account cannot be destroyed.
	ConsumerRemaining,
	/// There are no providers so the account cannot be created.
	NoProviders,
	/// There are too many consumers so the account cannot be created.
	TooManyConsumers,
	/// An error to do with tokens.
	Token,
	/// An arithmetic error.
	Arithmetic,
	/// The number of transactional layers has been reached, or we are not in a transactional
	/// layer.
	Transactional,
	/// Resources exhausted, e.g. attempt to read/write data which is too large to manipulate.
	Exhausted,
	/// The state is corrupt; this is generally not going to fix itself.
	Corruption,
	/// Some resource (e.g. a preimage) is unavailable right now. This might fix itself later.
	Unavailable,
	/// Root origin is not allowed.
	RootNotAllowed,
	/// An error with tries.
	Trie,
}

/// Reason why a pallet call failed.
#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
#[derive(Clone, ink::SolDecode, ink::SolEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
pub struct ModuleError {
	/// Module index, matching the metadata module index.
	pub index: u8,
	/// Module specific error value.
	pub error: FixedBytes<4>,
}

/// Description of what went wrong when trying to complete an operation on a token.
#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
#[derive(Copy, Clone, ink::SolDecode, ink::SolEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
pub enum TokenError {
	/// Funds are unavailable.
	FundsUnavailable,
	/// Some part of the balance gives the only provider reference to the account and thus cannot
	/// be (re)moved.
	OnlyProvider,
	/// Account cannot exist with the funds that would be given.
	BelowMinimum,
	/// Account cannot be created.
	CannotCreate,
	/// The token in question is unknown.
	Unknown,
	/// Funds exist but are frozen.
	Frozen,
	/// Operation is not supported by the token.
	Unsupported,
	/// Account cannot be created for a held balance.
	CannotCreateHold,
	/// Withdrawal would cause unwanted loss of account.
	NotExpendable,
	/// Account cannot receive the tokens.
	Blocked,
}

/// @Errors related to transactional storage layers.
#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
#[derive(Copy, Clone, ink::SolDecode, ink::SolEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
pub enum TransactionalError {
	/// Too many transactional layers have been spawned.
	LimitReached,
	/// A transactional layer was expected, but does not exist.
	NoLayer,
}

/// A runtime friendly error type for tries.
#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
#[derive(Copy, Clone, ink::SolDecode, ink::SolEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
pub enum TrieError {
	/// Attempted to create a trie with a state root not in the DB.
	InvalidStateRoot,
	/// Trie item not found in the database,
	IncompleteDatabase,
	/// A value was found in the trie with a nibble key that was not byte-aligned.
	ValueAtIncompleteKey,
	/// Corrupt Trie item.
	DecoderError,
	/// Hash is not value.
	InvalidHash,
	/// The statement being verified contains multiple key-value pairs with the same key.
	DuplicateKey,
	/// The proof contains at least one extraneous node.
	ExtraneousNode,
	/// The proof contains at least one extraneous value which should have been omitted from the
	/// proof.
	ExtraneousValue,
	/// The proof contains at least one extraneous hash reference the should have been omitted.
	ExtraneousHashReference,
	/// The proof contains an invalid child reference that exceeds the hash length.
	InvalidChildReference,
	/// The proof indicates that an expected value was not found in the trie.
	ValueMismatch,
	/// The proof is missing trie nodes required to verify.
	IncompleteProof,
	/// The root hash computed from the proof is incorrect.
	RootMismatch,
	/// One of the proof nodes could not be decoded.
	DecodeError,
}
