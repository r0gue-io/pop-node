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
			events::{Created, Transfer},
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
	}

	#[derive(scale::Decode, scale::Encode)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
	pub struct Member {
		voting_power: Balance,
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

		#[ink(message)]
		pub fn create_proposal(
			&mut self,
			beneficiary: AccountId,
			amount: Balance,
			description: String,
		) -> Result<(), Error> {
			let _caller = self.env().caller();
			let current_block = self.env().block_number();
			let proposal = Proposal {
				description,
				vote_start: current_block,
				vote_end: current_block + self.voting_period,
				yes_votes: 0,
				no_votes: 0,
				executed: false,
				beneficiary,
				amount,
			};

			self.proposals.push(proposal);
			Ok(())
		}

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

				if let Some(proposal) = self.proposals.get_mut(proposal_id_usize) {
					proposal.executed = true;
				}
				Ok(())
			} else {
				Err(Error::ProposalRejected)
			}
		}

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
}
