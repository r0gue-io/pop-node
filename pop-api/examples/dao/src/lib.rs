#[cfg(test)]
#[ink::contract]
mod dao {
	use ink::{
		prelude::{string::String, vec::Vec},
		storage::Mapping,
	};
	use pop_api::{
		primitives::TokenId,
		v0::fungibles::{
			self as api,
			events::{Approval, Created, Transfer},
			Psp22Error,
		},
	};

	/// Structure of the proposal used by the Dao governance sysytem
	#[derive(scale::Decode, scale::Encode, Debug)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub struct Proposal {
		// Description of the proposal
		description: String,

		// Beginnning of the voting period for this proposal
		vote_start: BlockNumber,

		// End of the voting period for this proposal
		vote_end: BlockNumber,

		// Balance representing the total votes for this proposal
		yes_votes: Balance,

		// Balance representing the total votes against this proposal
		no_votes: Balance,

		// Flag that indicates if the proposal was executed
		executed: bool,

		// AccountId of the recipient of the proposal
		beneficiary: AccountId,

		// Amount of tokens to be awarded to the beneficiary
		amount: Balance,

		// Identifier of the proposal
		proposal_id: u32,
	}

	/// Representation of a member in the voting system
	#[derive(scale::Decode, scale::Encode)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
	pub struct Member {
		// Stores the member's voting influence by using his balance
		voting_power: Balance,

		// Keeps track of the last vote casted by the member
		last_vote: BlockNumber,
	}

	/// Structure of a DAO (Decentralized Autonomous Organization)
	/// that uses Psp22 to manage the Dao treasury and funds projects
	/// selected by the members through governance
	#[ink(storage)]
	pub struct Dao {
		// Funding proposals
		proposals: Vec<Proposal>,

		// Mapping of AccountId to Member structs, representing DAO membership.
		members: Mapping<AccountId, Member>,

		// Mapping tracking the last time each account voted.
		last_votes: Mapping<AccountId, Timestamp>,

		// Duration of the voting period
		voting_period: BlockNumber,

		// Identifier of the Psp22 token associated with this DAO
		token_id: TokenId,
	}

	impl Dao {
		
		/// Instantiate a new Dao contract and create the associated token
		///
		/// # Parameters:
		/// - `token_id` - The identifier of the token to be created
		/// - `voting_period` - Amount of blocks during which members can cast their votes
		/// - `min_balance` - The minimum balance required for accounts holding this token.
		// The `min_balance` ensures accounts hold a minimum amount of tokens, preventing tiny,
		// inactive balances from bloating the blockchain state and slowing down the network.
		#[ink(constructor, payable)]
		pub fn new(
			token_id: TokenId,
			voting_period: BlockNumber,
			min_balance: Balance,
		) -> Result<Self, Psp22Error> {
			let instance = Self {
				proposals: Vec::new(),
				members: Mapping::default(),
				last_votes: Mapping::default(),
				voting_period,
				token_id: token_id.clone(),
			};
			let contract_id = instance.env().account_id();
			api::create(token_id, contract_id, min_balance).map_err(Psp22Error::from)?;
			instance.env().emit_event(Created {
				id: token_id,
				creator: contract_id,
				admin: contract_id,
			});

			Ok(instance)
		}

		/// Allows members to create new spending proposals
		///
		/// # Parameters
		/// - `beneficiary` - The account that will receive the payment
		/// if the proposal is accepted.
		/// - `amount` - Amount requested for this proposal
		/// - `description` - Description of the proposal
		#[ink(message)]
		pub fn create_proposal(
			&mut self,
			beneficiary: AccountId,
			amount: Balance,
			description: String,
		) -> Result<(), Error> {
			let _caller = self.env().caller();
			let current_block = self.env().block_number();
			let proposal_id: u32 = self.proposals.len().try_into().unwrap_or(0u32);
			let proposal = Proposal {
				description,
				vote_start: current_block,
				vote_end: current_block + self.voting_period,
				yes_votes: 0,
				no_votes: 0,
				executed: false,
				beneficiary,
				amount,
				proposal_id,
			};

			self.proposals.push(proposal);
			Ok(())
		}

		/// Allows Dao's members to vote for a proposal
		///
		/// # Parameters
		/// - `proposal_id` - Identifier of the proposal
		/// - `approve` - Indicates whether the vote is in favor (true) or against (false) the
		///   proposal.
		#[ink(message)]
		pub fn vote(&mut self, proposal_id: u32, approve: bool) -> Result<(), Error> {
			let caller = self.env().caller();
			let current_block = self.env().block_number();

			let proposal =
				self.proposals.get_mut(proposal_id as usize).ok_or(Error::ProposalNotFound)?;

			if current_block < proposal.vote_start || current_block > proposal.vote_end {
				return Err(Error::VotingPeriodEnded);
			}

			let member = self.members.get(caller).ok_or(Error::NotAMember)?;

			if member.last_vote >= proposal.vote_start {
				return Err(Error::AlreadyVoted);
			}

			if approve {
				proposal.yes_votes += member.voting_power;
			} else {
				proposal.no_votes += member.voting_power;
			}

			self.members.insert(
				caller,
				&Member { voting_power: member.voting_power, last_vote: current_block },
			);

			Ok(())
		}

		/// Enact a proposal approved by the Dao members
		///
		/// # Parameters
		/// - `proposal_id` - Identifier of the proposal
		#[ink(message)]
		pub fn execute_proposal(&mut self, proposal_id: u32) -> Result<(), Error> {
			let vote_end = self
				.proposals
				.get(proposal_id as usize)
				.ok_or(Error::ProposalNotFound)?
				.vote_end;

			// Check the voting period
			if self.env().block_number() <= vote_end {
				return Err(Error::VotingPeriodNotEnded);
			}

			// If we've passed the checks, now we can mutably borrow the proposal
			let proposal_id_usize = proposal_id as usize;
			let proposal = self.proposals.get(proposal_id_usize).ok_or(Error::ProposalNotFound)?;

			if proposal.executed {
				return Err(Error::ProposalAlreadyExecuted);
			}

			if proposal.yes_votes > proposal.no_votes {
				let contract = self.env().account_id();
				// ToDo: Check that there is enough funds in the treasury
				// Execute the proposal
				api::transfer(self.token_id, proposal.beneficiary, proposal.amount)
					.map_err(Psp22Error::from)?;
				self.env().emit_event(Transfer {
					from: Some(contract),
					to: Some(proposal.beneficiary),
					value: proposal.amount,
				});
				self.env().emit_event(Approval {
					owner: contract,
					spender: contract,
					value: proposal.amount,
				});

				if let Some(proposal) = self.proposals.get_mut(proposal_id_usize) {
					proposal.executed = true;
				}
				Ok(())
			} else {
				Err(Error::ProposalRejected)
			}
		}

		/// Allows a user to become a member of the Dao
		/// by transferring some tokens to the DAO's treasury.
		/// The amount of tokens transferred will be stored as the
		/// voting power of this member.
		///
		/// # Parameters
		/// - `amount` - Balance transferred to the Dao and representing
		/// the voting power of the member.

		#[ink(message)]
		pub fn join(&mut self, amount: Balance) -> Result<(), Error> {
			let caller = self.env().caller();
			let contract = self.env().account_id();
			api::transfer_from(self.token_id, caller.clone(), contract.clone(), amount)
				.map_err(Psp22Error::from)?;
			self.env().emit_event(Transfer {
				from: Some(caller),
				to: Some(contract),
				value: amount,
			});

			let member =
				self.members.get(caller).unwrap_or(Member { voting_power: 0, last_vote: 0 });

			self.members.insert(
				caller,
				&Member { voting_power: member.voting_power + amount, last_vote: member.last_vote },
			);

			Ok(())
		}
	}

	#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub enum Error {
		ProposalNotFound,
		VotingPeriodEnded,
		NotAMember,
		AlreadyVoted,
		VotingPeriodNotEnded,
		ProposalAlreadyExecuted,
		ProposalRejected,
		Psp22(Psp22Error),
	}

	impl From<Psp22Error> for Error {
		fn from(error: Psp22Error) -> Self {
			Error::Psp22(error)
		}
	}

	impl From<Error> for Psp22Error {
		fn from(error: Error) -> Self {
			match error {
				Error::Psp22(psp22_error) => psp22_error,
				_ => Psp22Error::Custom(String::from("Unknown error")),
			}
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;
		const UNIT: Balance = 10_000_000_000;
		const INIT_AMOUNT: Balance = 100_000_000 * UNIT;
		const INIT_VALUE: Balance = 100 * UNIT;
		const AMOUNT: Balance = MIN_BALANCE * 4;
		const MIN_BALANCE: Balance = 10_000;
		const TOKEN: TokenId = 1;

		#[ink::test]
		fn test_join() {
			// Setup
			let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
			// Test joining the DAO
			ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

			if let Ok(mut dao) = Dao::new(TOKEN, 10, MIN_BALANCE) {
				// Give some tokens to Alice
				let _ = api::transfer(TOKEN, accounts.alice, INIT_AMOUNT);

				assert_eq!(dao.join(AMOUNT), Ok(()));
				// Verify member was added correctly
				let member = dao.members.get(accounts.alice).unwrap();
				assert_eq!(member.voting_power, AMOUNT);
				assert_eq!(member.last_vote, 0);
			}
		}
	}
}
