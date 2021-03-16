#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{
	codec::{Decode, Encode},
	decl_module, decl_storage, decl_event, decl_error, traits::Get,
	traits::{ReservableCurrency, ExistenceRequirement, Currency, },
	debug, ensure,
};

use sp_runtime::{
	traits::{AccountIdConversion},
	ModuleId,
};

use frame_system::ensure_signed;
use sp_std::prelude::*;
use sp_std::{convert::{TryInto}};
use integer_sqrt::IntegerSquareRoot;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
	// used to generate sovereign account
	// refer: https://github.com/paritytech/substrate/blob/743accbe3256de2fc615adcaa3ab03ebdbbb4dbd/frame/treasury/src/lib.rs#L92
	type ModuleId: Get<ModuleId>;

	/// The currency in which the crowdfunds will be denominated
	type Currency: ReservableCurrency<Self::AccountId>;

	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

/// Simple index for identifying a fund.
pub type GrantIndex = u32;
pub type GrantRoundIndex = u32;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
type GrantOf<T> = Grant<AccountIdOf<T>>;
type ContributionOf<T> = Contribution<AccountIdOf<T>, BalanceOf<T>>;
type GrantRoundOf<T> = GrantRound<AccountIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;
type GrantInRoundOf<T> = GrantInRound<AccountIdOf<T>, BalanceOf<T>>;

/// Grant Round struct
#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Debug)]
pub struct GrantRound<AccountId, Balance, BlockNumber> {
	start: BlockNumber,
	end: BlockNumber,
	matching_fund: Balance,
	grants: Vec<GrantInRound<AccountId, Balance>>,
}

// Grant in round
#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Debug)]
pub struct GrantInRound<AccountId, Balance> {
	grant_index: GrantIndex,
	contributions: Vec<Contribution<AccountId, Balance>>,
	is_distributed_fund: bool,
}

/// Grant struct
#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Debug)]
pub struct Contribution<AccountId, Balance> {
	account_id: AccountId,
	value: Balance,
}

/// Grant struct
#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Debug)]
pub struct Grant<AccountId> {
	name: Vec<u8>,
	logo: Vec<u8>,
	description: Vec<u8>,
	website: Vec<u8>,
	/// The account that will receive the funds if the campaign is successful
	owner: AccountId,
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Config> as OpenGrantModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Grants get(fn grants): map hasher(blake2_128_concat) GrantIndex => Option<GrantOf<T>>;
		GrantCount get(fn fund_count): GrantIndex;

		GrantRounds get(fn grant_rounds): map hasher(blake2_128_concat) GrantRoundIndex => Option<GrantRoundOf<T>>;
		GrantRoundCount get(fn grant_round_count): GrantRoundIndex;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where Balance = BalanceOf<T>, AccountId = <T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, AccountId),
		GrantStored(GrantIndex, AccountId),
		ContributeSucceed(AccountId, u128),
		Contributed(AccountId, GrantIndex, Balance, BlockNumber),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// There was an overflow.
		Overflow,
		///
		RoundProcessing,
		StartBlockNumberInvalid,
		EndBlockNumberInvalid,
		EndTooEarly,
		NoActiveRound,
		NoActiveGrant,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// Create project
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn create_grant(origin, name: Vec<u8>, logo: Vec<u8>, description: Vec<u8>, website: Vec<u8>) {
			let who = ensure_signed(origin)?;

			// TODO: Validation
			let index = GrantCount::get();
			let next_index = index.checked_add(1).ok_or(Error::<T>::Overflow)?;

			// Create a grant 
			let grant = GrantOf::<T> {
				name: name,
				logo: logo,
				description: description,
				website: website,
				owner: who,
			};

			// Add grant to list
			<Grants<T>>::insert(index, grant);
			GrantCount::put(next_index);
		}

		/// Schedule a round
		/// If the last round is not over, no new round can be scheduled
		/// grant_indexes: the grants were selected for this round
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn schedule_round(origin, matching_fund: BalanceOf<T>, start: T::BlockNumber, end: T::BlockNumber, grant_indexes: Vec<GrantIndex>) {
			let who = ensure_signed(origin)?;
			let now = <frame_system::Module<T>>::block_number();
			let index = GrantCount::get();

			// The end block must be greater than the start block
			ensure!(end > start, Error::<T>::EndTooEarly);
			// Both the starting block number and the ending block number must be greater than the current number of blocks
			ensure!(start > now, Error::<T>::StartBlockNumberInvalid);
			ensure!(end > now, Error::<T>::EndBlockNumberInvalid);
			// Make sure the last round is over
			if index != 0 {
				let round = <GrantRounds<T>>::get(index).unwrap();
				ensure!(now > round.end, Error::<T>::RoundProcessing);
			}
			
			let next_index = index.checked_add(1).ok_or(Error::<T>::Overflow)?;

			// TODO: OOD
			// Create round
			let mut round = GrantRoundOf::<T> {
				start: start,
				end: end,
				matching_fund: matching_fund,
				grants: Vec::new(),
			};

			// Fill in the grants structure in advance
			for grant_index in grant_indexes {
				round.grants.push(GrantInRound {
					grant_index: grant_index,
					contributions: Vec::new(),
					is_distributed_fund: false,
				});
			}

			// Add grant round to list
			<GrantRounds<T>>::insert(index, round);
			GrantCount::put(next_index);

			// Transfer matching fund to module account
			T::Currency::transfer(
				&who,
				&Self::account_id(),
				matching_fund,
				ExistenceRequirement::AllowDeath
			)?;
			
			// 看看有没有插入成功
			let grant_round = <GrantRounds<T>>::get(index);
			debug::debug!("grant: {:#?}", grant_round);
		}

		/// Contribute a grant
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn contribute(origin, index: GrantIndex, value: BalanceOf<T>) {
			let who = ensure_signed(origin)?;
			let now = <frame_system::Module<T>>::block_number();
			
			// round list must be not none
			let round_index = GrantCount::get() - 1;
			ensure!(round_index > 0, Error::<T>::NoActiveRound);
			// The round must be in progress
			let mut round = <GrantRounds<T>>::get(round_index).ok_or(Error::<T>::NoActiveRound)?;
			ensure!(round.end < now, Error::<T>::NoActiveRound);

			// Find grant by index
			let mut found_grant: Option<&mut GrantInRoundOf::<T>> = None;
			for grant in round.grants.iter_mut() {
				if grant.grant_index == index {
					found_grant = Some(grant);
					break;
				}
			}

			// Find previous contribution by account_id
			// If you have contributed before, then add to that contribution. Otherwise join the list.
			let mut found_contribution: Option<&mut ContributionOf::<T>> = None;
			match found_grant {
				None => Err(Error::<T>::NoActiveGrant)?,
				Some(ref mut grant) => {
					for contribution in grant.contributions.iter_mut() {
						debug::debug!("contribution.account_id: {:#?}", contribution.account_id);
						debug::debug!("who: {:#?}", who);
						if contribution.account_id == who {
							found_contribution = Some(contribution);
							break;
						}
					}
					match found_contribution {
						Some(contribution) => {
							contribution.value += value;
							debug::debug!("contribution.value: {:#?}", contribution.value);
						},
						None => {
							grant.contributions.push(ContributionOf::<T> {
								account_id: who.clone(),
								value: value,
							});
							debug::debug!("contributions: {:#?}", grant.contributions);
						}
					}

					// Transfer contribute to grant account
					T::Currency::transfer(
						&who,
						&Self::grant_account_id(index),
						value,
						ExistenceRequirement::AllowDeath
					)?;
				},
			}
		}

		// Distribute fund from grant
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn distribute_fund(origin, round_index: GrantRoundIndex, grant_index: GrantIndex) {
			ensure!(round_index > 0, Error::<T>::NoActiveRound);
			let round = <GrantRounds<T>>::get(round_index).ok_or(Error::<T>::NoActiveRound)?;
			let mut grants = round.grants;

			// The round must have ended
			let now = <frame_system::Module<T>>::block_number();
			ensure!(round.end < now, Error::<T>::NoActiveRound);

			// Calculate CLR(Capital-constrained Liberal Radicalism) for grant
			let mut grant_clrs = Vec::new();
			let mut total_clr = 0;
			let matching_fund = Self::balance_to_u128(round.matching_fund);

			let mut found_grant: Option<&mut GrantInRoundOf::<T>> = None;
			let mut contribution_amount = 0;

			// Calculate grant CLR
			for grant in grants.iter_mut() {
				let mut sqrt_sum = 0;
				for contribution in grant.contributions.iter_mut() {
					let contribution_value = Self::balance_to_u128(contribution.value);
					debug::debug!("contribution_value: {}", contribution_value);
					sqrt_sum += contribution_value.integer_sqrt();
					contribution_amount += contribution_value;
				}
				debug::debug!("grant_sum: {}", sqrt_sum);
				let grant_clr = sqrt_sum * sqrt_sum;
				grant_clrs.push(grant_clr);
				total_clr += grant_clr;

				if grant.grant_index == grant_index {
					found_grant = Some(grant);
				}
			}

			let grant = found_grant.ok_or(Error::<T>::NoActiveGrant)?;

			// This grant must not have distributed funds
			ensure!(grant.is_distributed_fund, Error::<T>::NoActiveGrant);

			// Calculate CLR
			let grant_clr = grant_clrs[grant_index as usize];
			let project = Grants::<T>::get(grant_index).ok_or(Error::<T>::NoActiveGrant)?;
			let grant_matching_fund = ((grant_clr as f64 / total_clr as f64) * matching_fund as f64) as u128;

			// Distribute CLR amount
			T::Currency::transfer(
				&Self::account_id(),
				&project.owner,
				Self::u128_to_balance(grant_matching_fund),
				ExistenceRequirement::AllowDeath
			)?;

			// Distribute distribution
			let grant_account_id = &Self::grant_account_id(grant_index as u32);
			T::Currency::transfer(
				grant_account_id,
				&project.owner,
				Self::u128_to_balance(contribution_amount),
				ExistenceRequirement::AllowDeath
			)?;

			// Set is_distributed_fund
			grant.is_distributed_fund = true;
		}
	}
}

impl<T: Config> Module<T> {
	/// The account ID of the fund pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn account_id() -> T::AccountId {
		let account = T::ModuleId::get().into_account();
		// println!("account: {}", account);
		return account;
	}

	pub fn grant_account_id(index: GrantIndex) -> T::AccountId {
		T::ModuleId::get().into_sub_account(index)
	}

	pub fn u128_to_balance(cost: u128) -> BalanceOf<T> {
		TryInto::<BalanceOf::<T>>::try_into(cost).ok().unwrap()
	}

	pub fn balance_to_u128(balance: BalanceOf<T>) -> u128 {
		TryInto::<u128>::try_into(balance).ok().unwrap()
	}

}