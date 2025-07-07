#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{env::Error as EnvError, prelude::vec::Vec, storage::Mapping};
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

	#[ink::scale_derive(Encode)]
	pub enum RuntimeCall {
		/// We can add additional pallets we might want to use here
		#[codec(index = 150)]
		Fungibles(FungiblesCall),
	}

	#[ink::scale_derive(Encode)]
	pub enum FungiblesCall {
		#[codec(index = 4)]
		TransferFrom { token: TokenId, from: AccountId, to: AccountId, value: Balance },
	}

	/// Structure of the proposal used by the Dao governance sysytem
	#[derive(Debug, Clone)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
	pub struct Proposal {
		/// Description of the proposal
		pub description: Vec<u8>,
		/// Flag that indicates if the proposal was Executed
		pub status: ProposalStatus,
		/// Identifier of the proposal
		pub proposal_id: u32,
		/// Information related to voting
		pub round: Option<VoteRound>,

		// Information relative to proposal execution if approved
		pub transaction: Option<Transaction>,
	}

	impl Default for Proposal {
		fn default() -> Self {
			let fetch_dao = ink::env::get_contract_storage::<u32, Dao>(&0u32)
				.expect("The dao should have been created already");

			// The dao is supposed to exist at this point
			let dao = fetch_dao.unwrap_or_default();
			let voting_period = dao.voting_period;
			let current_block = ink::env::block_number::<Environment>();
			let end = current_block.saturating_add(voting_period);
			let round =
				Some(VoteRound { start: current_block, end, yes_votes: 0, no_votes: 0 });
			Proposal {
				description: Vec::new(),
				status: ProposalStatus::Submitted,
				proposal_id: 0,
				round,
				transaction: None,
			}
		}
	}

	/// Representation of a member in the voting system
	#[derive(Debug, Clone, Default)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
	pub struct Member {
		// Stores the member's voting influence by using his balance
		pub voting_power: Balance,

		// Keeps track of the last vote casted by the member
		pub last_vote: BlockNumber,
	}

	#[derive(Debug, Clone)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
	pub struct VoteRound {
		// Beginnning of the voting period for this proposal
		pub start: BlockNumber,

		// End of the voting period for this proposal
		pub end: BlockNumber,

		// Balance representing the total votes for this proposal
		pub yes_votes: Balance,

		// Balance representing the total votes against this proposal
		pub no_votes: Balance,
	}

	impl VoteRound {
		fn get_status(&self, mut proposal: Proposal) -> Proposal {
			if proposal.status == ProposalStatus::Submitted {
				if self.yes_votes > self.no_votes {
					proposal.status = ProposalStatus::Approved;
				} else {
					proposal.status = ProposalStatus::Rejected;
				}
			};
			proposal
		}

		fn update_votes(&mut self, approved: bool, member: Member) {
			match approved {
				true => {
					self.yes_votes =
						self.yes_votes.saturating_add(member.voting_power);
				},
				false => {
					self.no_votes = self.no_votes.saturating_add(member.voting_power);
				},
			};
		}
	}

	#[derive(Debug, Clone)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
	pub struct Transaction {
		// The recipient of the proposal
		pub beneficiary: AccountId,
		// Amount of tokens to be awarded to the beneficiary
		pub amount: Balance,
	}

	/// Structure of a DAO (Decentralized Autonomous Organization)
	/// that uses Psp22 to manage the Dao treasury and funds projects
	/// selected by the members through governance
	#[derive(Default)]
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
		proposal_count: u32,
	}

	/// Defines an event that is emitted
	/// every time a member voted.
	#[derive(Debug)]
	#[ink(event)]
	pub struct Vote {
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
				proposal_count: 0,
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
			mut description: Vec<u8>,
		) -> Result<(), Error> {
			let caller = self.env().caller();
			let contract = self.env().account_id();
			

			if description.len() >= u8::MAX.into() {
				return Err(Error::MaxDescriptionLengthReached);
			}

			self.proposal_count = self.proposal_count.saturating_add(1);
			let mut proposal =
				Proposal { proposal_id: self.proposal_count, ..Default::default() };
			proposal.description.append(&mut description);
			let transaction = Transaction { beneficiary, amount };
			proposal.transaction = Some(transaction);

			self.proposals.insert(proposal.proposal_id, &proposal);

			self.env().emit_event(Created {
				id: proposal.proposal_id,
				creator: caller,
				admin: contract,
			});

			Ok(())
		}

		/// Vote on a proposal. Only members can vote.
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
			let mut round = proposal.round.clone().ok_or(Error::ProblemWithTheContract)?;

			if current_block > round.end {
				// Update the Proposal status if needed
				proposal = round.get_status(proposal);
				self.proposals.insert(proposal.proposal_id, &proposal);
				return Err(Error::VotingPeriodEnded);
			}

			let member = self.members.get(caller).ok_or(Error::MemberNotFound)?;

			if member.last_vote >= round.start {
				return Err(Error::AlreadyVoted);
			}

			round.update_votes(approve, member.clone());
			proposal.round = Some(round);

			self.proposals.insert(proposal_id, &proposal);

			self.members.insert(
				caller,
				&Member { voting_power: member.voting_power, last_vote: current_block },
			);
			self.last_votes.insert(caller, &current_block);

			self.env().emit_event(Vote { who: Some(caller), when: Some(current_block) });

			Ok(())
		}

		/// Enact a proposal Approved by the Dao members
		///
		/// # Parameters
		/// - `proposal_id` - Identifier of the proposal
		#[ink(message)]
		pub fn execute_proposal(&mut self, proposal_id: u32) -> Result<(), Error> {
			let mut proposal = self.proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;
			let round = proposal.round.clone().ok_or(Error::ProblemWithTheContract)?;

			let transaction =
				proposal.transaction.clone().ok_or(Error::ProblemWithTheContract)?;

			// Check the voting period
			if self.env().block_number() <= round.end {
				return Err(Error::VotingPeriodNotEnded);
			}

			if proposal.status == ProposalStatus::Executed {
				return Err(Error::ProposalExecuted);
			}

			if round.yes_votes > round.no_votes {
				let contract = self.env().account_id();

				// Execute the proposal
				let _treasury_balance = match api::balance_of(self.token_id, contract) {
					Ok(val) if val > transaction.amount => val,
					_ => {
						return Err(Error::NotEnoughFundsAvailable);
					},
				};

				// RuntimeCall.
				let _ = self.env()
					.call_runtime(&RuntimeCall::Fungibles(FungiblesCall::TransferFrom {
						token: self.token_id,
						from: contract,
						to: transaction.beneficiary,
						value: transaction.amount,
					}))
					.map_err(EnvError::from);

				self.env().emit_event(Transfer {
					from: Some(contract),
					to: Some(transaction.beneficiary),
					value: transaction.amount,
				});
				self.env().emit_event(Approval {
					owner: contract,
					spender: contract,
					value: transaction.amount,
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
				self.members.get(caller).unwrap_or_default();

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
			self.members.get(account).unwrap_or_default()
		}

		#[ink(message)]
		pub fn get_proposal(&mut self, proposal_id: u32) -> Option<Proposal> {
			self.proposals.get(proposal_id)
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
		MaxDescriptionLengthReached,

		/// There are not enough funds in the Dao treasury
		NotEnoughFundsAvailable,

		/// The contract creation failed, a new contract is needed
		ProblemWithTheContract,

		/// The Runtime Call failed
		ProposalExecutionFailed,

		/// PSP22 specific error
		Psp22(Psp22Error),
	}

	impl From<Psp22Error> for Error {
		fn from(error: Psp22Error) -> Self {
			Self::Psp22(error)
		}
	}

	impl From<EnvError> for Error {
		fn from(e: EnvError) -> Self {
			use ink::env::ReturnErrorCode;
			match e {
				EnvError::ReturnError(ReturnErrorCode::CallRuntimeFailed) =>
					Error::ProposalExecutionFailed,
				_ => panic!("Unexpected error from `pallet-contracts`."),
			}
		}
	}
}
