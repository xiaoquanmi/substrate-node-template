#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use codec::{Codec, MaxEncodedLen};
	use core::fmt::Debug;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, Randomness, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, One};
	use sp_std::vec::Vec;

	// type KittyIndex = u32;

	// #[pallet::type_value]
	// pub fn GetDefaultValue() -> KittyIndex {
	// 	0_u32
	// }

	#[pallet::type_value]
	pub fn GetDefaultValue<T: Config>() -> T::KittyIndex {
		T::KittyIndex::default()
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		type StakeForEachKitty: Get<BalanceOf<Self>>;

		/// Kitty Index
		type KittyIndex: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Codec
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaxEncodedLen
			+ TypeInfo;
	}

	#[pallet::pallet]
	// FIXME: ?
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> =
		StorageValue<_, T::KittyIndex, ValueQuery, GetDefaultValue<T>>;
	// pub type NextKittyId<T: Config> = StorageValue<_, T::KittyIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn owner_2_kitties)]
	pub type Owner2Kitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::KittyIndex>>;

	#[pallet::storage]
	#[pallet::getter(fn price)]
	pub type Price<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<BalanceOf<T>>, ValueQuery>;

	// associated type `AccountId` not found
	// pub type KittyOwner<T> = StorageMap<_, Blake2_128Concat, KittyIndex, T::AccountId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, T::KittyIndex, Kitty),
		KittyBread(T::AccountId, T::KittyIndex, Kitty),
		KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),
		KittySaled(T::AccountId, T::KittyIndex, Option<BalanceOf<T>>),
		KittySold(T::AccountId, T::AccountId, T::KittyIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		NotOwner,
		IsOwner,
		SameKittyId,
		InsufficientBalance,
		MaxLenKitties,
		KittyNotForSell,
		NotEnoughBalanceForBuying,
		NotEnoughBalanceForStaking,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

			// Reserve for create kitty
			let amount = T::StakeForEachKitty::get();
			T::Currency::reserve(&who, amount).map_err(|_| Error::<T>::InsufficientBalance)?;

			let dna = Self::random_value(&who);
			let kitty = Kitty(dna);

			// add kitty id to owner
			if Owner2Kitties::<T>::contains_key(&who) {
				Owner2Kitties::<T>::mutate(
					&who,
					|value| -> Result<(), sp_runtime::DispatchError> {
						if let Some(v) = value {
							v.push(kitty_id)
						}
						Ok(())
					},
				)?
			} else {
				let mut value: Vec<T::KittyIndex> = Vec::new();
				value.push(kitty_id);
				Owner2Kitties::<T>::insert(&who, &value);
			}

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			// NextKittyId::<T>::set(kitty_id + 1);
			NextKittyId::<T>::set(kitty_id + T::KittyIndex::one());

			Self::deposit_event(Event::KittyCreated(who, kitty_id, kitty));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn bread(
			origin: OriginFor<T>,
			kitty_id_1: T::KittyIndex,
			kitty_id_2: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// check kitty id
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);
			let kitty_1 = Self::get_kitty(kitty_id_1).map_err(|_| Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::get_kitty(kitty_id_2).map_err(|_| Error::<T>::InvalidKittyId)?;

			// Reserve for create kitty
			let amount = T::StakeForEachKitty::get();
			T::Currency::reserve(&who, amount).map_err(|_| Error::<T>::InsufficientBalance)?;

			// get next kitty id
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

			// selector for breeding
			let selector = Self::random_value(&who);

			let mut data = [0u8; 16];

			for i in 0..kitty_1.0.len() {
				data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & !selector[i]);
			}

			let new_kitty = Kitty(data);

			Kitties::<T>::insert(kitty_id, &new_kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			NextKittyId::<T>::set(kitty_id + T::KittyIndex::one());

			Self::deposit_event(Event::KittyBread(who, kitty_id, new_kitty));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;

			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);

			// 质押的金额
			let stake_amount = T::StakeForEachKitty::get();
			// 转移质押
			T::Currency::transfer(&who, &new_owner, stake_amount, ExistenceRequirement::KeepAlive)?;
			T::Currency::reserve(&new_owner, stake_amount)
				.map_err(|_| Error::<T>::NotEnoughBalanceForStaking)?;

			KittyOwner::<T>::insert(kitty_id, &new_owner);
			// <KittyOwner::<T>>::insert(kitty_id, &new_owner);

			Self::deposit_event(Event::KittyTransferred(who, new_owner, kitty_id));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);

			payload.using_encoded(blake2_128)
		}

		fn get_next_id() -> Result<T::KittyIndex, ()> {
			match Self::next_kitty_id() {
				// T::KittyIndex::MAX => Err(()),
				// T::KittyIndex::max_value() => Err(()),
				// | ^^^^^^^^^^^^^^^^^^^^^^^^^^ `fn` calls are not allowed in patterns
				val if val == T::KittyIndex::max_value() => Err(()),
				val => Ok(val),
			}
		}

		fn get_kitty(kitty_id: T::KittyIndex) -> Result<Kitty, ()> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err(()),
			}
		}
	}
}
