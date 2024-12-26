#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{prelude::vec::Vec, storage::Mapping};
use pop_api::{
	primitives::TokenId,
	v0::fungibles::{
		self as api,
		events::{Approval, Created, Transfer},
		Psp22Error,
	},
};

#[cfg(test)]
mod tests;

#[ink::contract]
mod dao {
	use super::*;

	#[derive(Debug, Clone, PartialEq)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
	pub enum ProposalStatus {
		Submitted,
		Approved,
		Rejected,
		Executed,
	}

	/// Structure of the proposal used by the Dao governance sysytem
	#[derive(Debug, Clone)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
	pub struct Proposal {
		// Description of the proposal
		pub description: Vec<u8>,

		// Beginnning of the voting period for this proposal
		pub vote_start: BlockNumber,

		// End of the voting period for this proposal
		pub vote_end: BlockNumber,

		// Balance representing the total votes for this proposal
		pub yes_votes: Balance,

		// Balance representing the total votes against this proposal
		pub no_votes: Balance,

		// Flag that indicates if the proposal was Executed
		pub status: ProposalStatus,

		// The recipient of the proposal
		pub beneficiary: AccountId,

		// Amount of tokens to be awarded to the beneficiary
		pub amount: Balance,

		// Identifier of the proposal
		pub proposal_id: u32,
	}

	/// Representation of a member in the voting system
	#[derive(Debug)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
	pub struct Member {
		// Stores the member's voting influence by using his balance
		pub voting_power: Balance,

		// Keeps track of the last vote casted by the member
		pub last_vote: BlockNumber,
	}

	/// Structure of a DAO (Decentralized Autonomous Organization)
	/// that uses Psp22 to manage the Dao treasury and funds projects
	/// selected by the members through governance
	#[ink(storage)]
	pub struct Dao {
		// Funding proposals
		proposals: Mapping<u32, Proposal>,

		// Mapping of AccountId to Member structs, representing DAO membership.
		members: Mapping<AccountId, Member>,

		// Mapping tracking the last time each account voted.
		last_votes: Mapping<AccountId, BlockNumber>,

		// Duration of the voting period
		voting_period: BlockNumber,

		// Identifier of the Psp22 token associated with this DAO
		token_id: TokenId,

		// Proposals created in the history of the Dao
		proposals_created: u32,
	}

	/// Defines an event that is emitted
	/// every time a member voted.
	#[derive(Debug)]
	#[ink(event)]
	pub struct Voted {
		pub who: Option<AccountId>,
		pub when: Option<BlockNumber>,
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
				proposals: Mapping::default(),
				members: Mapping::default(),
				last_votes: Mapping::default(),
				voting_period,
				token_id,
				proposals_created: 0,
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
			description: Vec<u8>,
		) -> Result<(), Error> {
			let caller = self.env().caller();
			let contract = self.env().account_id();
			let current_block = self.env().block_number();
			let proposal_id: u32 = self.proposals_created.saturating_add(1);
			let vote_end = current_block.saturating_add(self.voting_period);
			if description.len() >= u8::MAX.into() {
				return Err(Error::ExceedeMaxDescriptionLength);
			}
			let proposal = Proposal {
				description,
				vote_start: current_block,
				vote_end,
				yes_votes: 0,
				no_votes: 0,
				status: ProposalStatus::Submitted,
				beneficiary,
				amount,
				proposal_id,
			};

			self.proposals.insert(proposal_id, &proposal);

			self.env()
				.emit_event(Created { id: proposal_id, creator: caller, admin: contract });

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
			let mut proposal = self.proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;

			if current_block > proposal.vote_end {
				if proposal.status == ProposalStatus::Submitted {
					if proposal.yes_votes > proposal.no_votes {
						proposal.status = ProposalStatus::Approved;
					} else {
						proposal.status = ProposalStatus::Rejected;
					}
				};

				return Err(Error::VotingPeriodEnded);
			}

			let member = self.members.get(caller).ok_or(Error::MemberNotFound)?;

			if member.last_vote >= proposal.vote_start {
				return Err(Error::AlreadyVoted);
			}

			let votes = match approve {
				true => proposal.yes_votes,
				false => proposal.no_votes,
			};

			let _ = votes.saturating_add(member.voting_power);

			self.proposals.insert(proposal_id, &proposal);

			self.members.insert(
				caller,
				&Member { voting_power: member.voting_power, last_vote: current_block },
			);
			self.last_votes.insert(caller, &current_block);

			self.env().emit_event(Voted { who: Some(caller), when: Some(current_block) });

			Ok(())
		}

		/// Enact a proposal Approved by the Dao members
		///
		/// # Parameters
		/// - `proposal_id` - Identifier of the proposal
		#[ink(message)]
		pub fn execute_proposal(&mut self, proposal_id: u32) -> Result<(), Error> {
			let mut proposal = self.proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;

			// Check the voting period
			if self.env().block_number() <= proposal.vote_end {
				return Err(Error::VotingPeriodNotEnded);
			}

			if proposal.status == ProposalStatus::Executed {
				return Err(Error::ProposalExecuted);
			}

			if proposal.yes_votes > proposal.no_votes {
				let contract = self.env().account_id();

				// Execute the proposal
				let _treasury_balance = match api::balance_of(self.token_id, contract) {
					Ok(val) if val > proposal.amount => val,
					_ => {
						return Err(Error::NotEnoughFundsAvailable);
					},
				};

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

				proposal.status = ProposalStatus::Executed;

				self.proposals.insert(proposal_id, &proposal);
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
			api::transfer_from(self.token_id, caller, contract, amount)
				.map_err(Psp22Error::from)?;
			let member =
				self.members.get(caller).unwrap_or(Member { voting_power: 0, last_vote: 0 });

			let voting_power = member.voting_power.saturating_add(amount);
			self.members
				.insert(caller, &Member { voting_power, last_vote: member.last_vote });

			self.env().emit_event(Transfer {
				from: Some(caller),
				to: Some(contract),
				value: amount,
			});

			Ok(())
		}

		#[ink(message)]
		pub fn get_member(&mut self, account: AccountId) -> Member {
			self.members.get(account).unwrap_or(Member { voting_power: 0, last_vote: 0 })
		}

		#[ink(message)]
		pub fn positive_votes(&mut self, proposal_id: u32) -> Option<Balance> {
			self.proposals.get(proposal_id).as_ref().map(|x| x.yes_votes)
		}

		#[ink(message)]
		pub fn negative_votes(&mut self, proposal_id: u32) -> Option<Balance> {
			self.proposals.get(proposal_id).as_ref().map(|x| x.no_votes)
		}
	}

	#[derive(Debug, PartialEq, Eq)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	pub enum Error {
		/// This proposal does not exists
		ProposalNotFound,

		/// The end of the voting period has been reached
		VotingPeriodEnded,

		/// User is not a member of this Dao
		MemberNotFound,

		/// User already voted for this proposal
		AlreadyVoted,

		/// The voting period for this proposal is still ongoing
		VotingPeriodNotEnded,

		/// This proposal has already been Executed
		ProposalExecuted,

		/// This proposal has been Rejected
		ProposalRejected,

		/// The proposal description is too long
		ExceedeMaxDescriptionLength,

		/// There are not enough funds in the Dao treasury
		NotEnoughFundsAvailable,

		/// PSP22 specific error
		Psp22(Psp22Error),
	}

	impl From<Psp22Error> for Error {
		fn from(error: Psp22Error) -> Self {
			Error::Psp22(error)
		}
	}

	// impl From<Error> for Psp22Error {
	// fn from(error: Error) -> Self {
	// match error {
	// Error::Psp22(psp22_error) => psp22_error,
	// _ => Psp22Error::Custom(String::from("Unknown error")),
	// }
	// }
	// }
}
