#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::{Randomness, Currency, ReservableCurrency, ExistenceRequirement}};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded};

	#[derive(Encode, Decode)]
	pub struct Kitty(pub [u8;16]);

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type KittyIndex: Parameter + AtLeast32BitUnsigned + Default + Copy + Bounded;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		type KittyDeposit: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, T::KittyIndex),
		KittyTransfer(T::AccountId, T::AccountId, T::KittyIndex),
		KittyOnSale(T::AccountId, T::KittyIndex, Option<BalanceOf<T>>),
		KittySold(T::AccountId, T::AccountId, T::KittyIndex, BalanceOf<T>),
	}

	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_price)]
	pub type KittiesPrice<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<BalanceOf<T>>, ValueQuery>;

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		KittiesCountOverflow,
		NotOwner,
		SameParentIndex,
		InvalidKittyIndex,
		NotForSale,
		NotEnoughBalance,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let kitty_id = Self::next_kitty_id()?;

			T::Currency::reserve(&who, T::KittyDeposit::get()).map_err(|_| Error::<T>::NotEnoughBalance)?;

			let dna = Self::random_value(&who);

			Self::insert_kitty(&who, kitty_id, Kitty(dna));

			Self::deposit_event(Event::KittyCreated(who, kitty_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(origin: OriginFor<T>, new_owner: T::AccountId, kitty_id: T::KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);
			Owner::<T>::insert(kitty_id, Some(new_owner.clone()));
			Self::deposit_event(Event::KittyTransfer(who, new_owner, kitty_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn breed(origin: OriginFor<T>, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);

			let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

			let kitty_id = Self::next_kitty_id()?;

			let dna_1 = kitty1.0;
			let dna_2 = kitty2.0;

			let selector = Self::random_value(&who);
			let mut new_dna = [0u8; 16];

			for i in 0..dna_1.len() {
				new_dna[i] = (selector[i] & dna_1[i]) | (selector[i] & dna_2[i]);
			}
			
			Self::insert_kitty(&who, kitty_id, Kitty(new_dna));
			Self::deposit_event(Event::KittyCreated(who, kitty_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn sell(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Some(who.clone()) == Self::owner(kitty_id), Error::<T>::NotOwner);

			KittiesPrice::<T>::mutate_exists(kitty_id, |p| *p = Some(price));

			Self::deposit_event(Event::KittyOnSale(who, kitty_id, price));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn buy(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let owner = Self::owner(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;

			let kitty_price = Self::kitties_price(kitty_id).ok_or(Error::<T>::NotForSale)?;

			let reserve = T::KittyDeposit::get();
           
			T::Currency::reserve(&who, reserve).map_err(|_| Error::<T>::NotEnoughBalance)?;
			T::Currency::unreserve(&owner, reserve);

			T::Currency::transfer(&who, &owner, kitty_price, ExistenceRequirement::KeepAlive)?;

			<KittiesPrice<T>>::remove(kitty_id);

			Owner::<T>::insert(kitty_id, Some(who.clone()));

			Self::deposit_event(Event::KittySold(owner, who, kitty_id, kitty_price));

			Ok(())
		}
	}

	impl<T:Config> Pallet<T> {
		fn random_value(sender: &T::AccountId) -> [u8;16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}

		fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
			Kitties::<T>::insert(kitty_id, Some(kitty));
			Owner::<T>::insert(kitty_id, Some(owner.clone()));
			KittiesCount::<T>::put(kitty_id + 1u32.into());
		}

		fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
			let kitty_id = match Self::kitties_count() {
				Some(id) => {
					if id == T::KittyIndex::max_value() {
						return Err(Error::<T>::KittiesCountOverflow.into());
					}
					id
				},
				None => {
					0u32.into()
				}
			};

			Ok(kitty_id)
		}
	}
}
