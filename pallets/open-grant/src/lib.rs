#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{
	codec::{Decode, Encode},
	decl_module, decl_storage, decl_event, decl_error, dispatch, traits::Get,
	traits::{ReservableCurrency, ExistenceRequirement, Currency, },
	storage::child,
	debug,
};

use sp_runtime::{
	traits::{AccountIdConversion, Zero, Saturating, },
	ModuleId,
};

use frame_system::ensure_signed;
use sp_std::prelude::*;
use sp_std::{convert::{TryInto}};
use sp_core::Hasher;
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

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
type GrantOf<T> = Grant<AccountIdOf<T>, BalanceOf<T>>;
type ContributionOf<T> = Contribution<AccountIdOf<T>, BalanceOf<T>>;


/// GrantRound struct
// #[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Debug)]
// pub struct GrantRound<BlockNumber> {
// 	start: BlockNumber,
// 	end: BlockNumber,
// }

/// Grant struct
#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Debug)]
pub struct Contribution<AccountId, Balance> {
	account_id: AccountId,
	value: Balance,
}

/// Grant struct
#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Debug)]
pub struct Grant<AccountId, Balance> {
	name: Vec<u8>,
	logo: Vec<u8>,
	description: Vec<u8>,
	website: Vec<u8>,
	/// The account that will receive the funds if the campaign is successful
	owner: AccountId,
	/// contributions
	contributions: Vec<Contribution<AccountId, Balance>>,
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
		Something get(fn something): Option<u32>;
		Grants get(fn projects): Vec<GrantOf<T>>;
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

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			Something::put(something);

			// Emit an event.
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			// Return a successful DispatchResult
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Something::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::put(new);
					Ok(())
				},
			}
		}

		/// Create grant
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn create_grant(origin, grant: GrantOf<T>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			let mut grants = Grants::<T>::get();
			grants.push(grant.clone());
			Grants::<T>::put(&grants);
			Self::deposit_event(RawEvent::GrantStored((grants.len() - 1).try_into().unwrap(), who));
			Ok(())
		}

		/// Create project
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn create_grant_plan(origin, name: Vec<u8>, logo: Vec<u8>, description: Vec<u8>, website: Vec<u8>) -> dispatch::DispatchResult {
			debug::RuntimeLogger::init();
			let who = ensure_signed(origin)?;
			let grant = GrantOf::<T> {
				name: name,
				logo: logo,
				description: description,
				website: website,
				owner: who.clone(),
				contributions: vec![],
			};
			let mut grants = Grants::<T>::get();
			grants.push(grant.clone());

			frame_support::debug::native::debug!("grant: {:#?}", grant);
			frame_support::debug::native::debug!("grants.len(): {}", grants.len());

			Grants::<T>::put(grants.clone());
			// Self::deposit_event(RawEvent::GrantStored((grants.len() - 1).try_into().unwrap(), who));
			Ok(())
		}

		/// hello
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn hello(origin) -> dispatch::DispatchResult {
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn contribute(origin, index: GrantIndex, value: BalanceOf<T>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			
			let now = <frame_system::Module<T>>::block_number();

			let mut grants = Grants::<T>::get();
			
			T::Currency::transfer(
				&who,
				&Self::grant_account_id(index),
				value,
				ExistenceRequirement::AllowDeath
			)?;

			debug::debug!("grants.len: {}", grants.len());
			debug::debug!("grants[index as usize].contributions.len: {}", grants[index as usize].contributions.len());

			let mut found_contribution: Option<&mut ContributionOf::<T>> = None;
			for contribution in grants[index as usize].contributions.iter_mut() {
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
				}
				None => {
					grants[index as usize].contributions.push(ContributionOf::<T> {
						account_id: who.clone(),
						value: value,
					});
					debug::debug!("contributions: {:#?}", grants[index as usize].contributions);
				}
			}

			Grants::<T>::put(grants.clone());

			// Self::deposit_event(RawEvent::Contributed(who, index, balance, now));

			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn start_round(origin, value: BalanceOf<T>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			T::Currency::transfer(
				&who,
				&Self::account_id(),
				value,
				ExistenceRequirement::AllowDeath
			)?;
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn end_round(origin) -> dispatch::DispatchResult {
			let mut grants = Grants::<T>::get();
			let mut grant_clrs = Vec::new();
			let mut total_clr = 0;
			for grant in grants.iter_mut() {
				debug::debug!("grantgrantgrantgrantgrantgrantgrantgrantgrant, {:#?}", grant);
				let mut sqrt_sum = 0;
				for contribution in grant.contributions.iter() {
					let contribution_value = Self::balance_to_u128(contribution.value);
					debug::debug!("contribution_value: {}", contribution_value);
					sqrt_sum += contribution_value.integer_sqrt();
				}
				debug::debug!("grant_sum: {}", sqrt_sum);
				let grant_clr = sqrt_sum * sqrt_sum;
				grant_clrs.push(grant_clr);
				total_clr += grant_clr;
			}

			let total = Self::balance_to_u128(T::Currency::total_balance(&Self::account_id()));
			let total_free = Self::balance_to_u128(T::Currency::free_balance(&Self::account_id()));
			debug::debug!("total: {}", total);
			debug::debug!("total_free: {}", total_free);

			for i in 0..grants.len() {
				let grant_clr = grant_clrs[i];
				let bal = Self::u128_to_balance(((grant_clr as f64 / total_clr as f64) * total as f64) as u128);
				debug::debug!("Self::account_id(): {:#?}", Self::account_id());
				debug::debug!("grants[i].owner: {:#?}", grants[i].owner);
				debug::debug!("bal: {:#?}", bal);
				T::Currency::transfer(
					&Self::account_id(),
					&grants[i].owner,
					Self::u128_to_balance(((grant_clr as f64 / total_clr as f64) * total as f64) as u128),
					ExistenceRequirement::AllowDeath
				)?;
			}

			// let total = T::Currency::total_balance(&Self::account_id());
			Ok(())
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