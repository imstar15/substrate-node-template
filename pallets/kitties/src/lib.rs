#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Randomness};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;

	#[derive(Encode, Decode)]
	pub struct Kitty(pub [u8;16]);

	type KittyIndex = u32;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
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
		KittyCreated(T::AccountId, KittyIndex),
		KittyTransfer(T::AccountId, T::AccountId, KittyIndex),
	}

	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type KittiesCount<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyIndex, Option<Kitty>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, KittyIndex, Option<T::AccountId>, ValueQuery>;

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		KittiesCountOverflow,
		NotOwner,
		SameParaentIndex,
		InvalidKittyIndex,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let kitty_id = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					id
				},
				None => {
					1
				}
			};

			let dna = Self::random_value(&who);

			Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			KittiesCount::<T>::put(kitty_id + 1);
			Self::deposit_event(Event::KittyCreated(who, kitty_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(origin: OriginFor<T>, new_owner: T::AccountId, kitty_id: KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);
			Owner::<T>::insert(kitty_id, Some(new_owner.clone()));
			Self::deposit_event(Event::KittyTransfer(who, new_owner, kitty_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn breed(origin: OriginFor<T>, kitty_id_1: KittyIndex, kitty_id_2: KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParaentIndex);

			let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

			let kitty_id = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					id
				},
				None => {
					1
				}
			};

			let dna_1 = kitty1.0;
			let dna_2 = kitty2.0;

			let selector = Self::random_value(&who);
			let mut new_dna = [0u8; 16];

			for i in 0..dna_1.len() {
				new_dna[i] = (selector[i] & dna_1[i]) | (selector[i] & dna_2[i]);
			}

			Kitties::<T>::insert(kitty_id, Some(Kitty(new_dna)));
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			KittiesCount::<T>::put(kitty_id + 1);
			Self::deposit_event(Event::KittyCreated(who, kitty_id));

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
	}
}
