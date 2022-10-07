use crate::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
// use frame_support::traits::Get;
use frame_system::RawOrigin;
// use sp_std::vec;

fn insert_claim<T: Config>(claim: &Vec<u8>, sender: &T::AccountId) {
	let bounded_claim =
		BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).expect("claim too long");
	Proofs::<T>::insert(
		&bounded_claim,
		(sender.clone(), frame_system::Pallet::<T>::block_number()),
	);
}

benchmarks! {
	create_claim {
		let d in 0 .. T::MaxClaimLength::get();
		let claim = vec![0; d as usize];
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller), claim)

	revoke_claim {
		let d in 0 .. T::MaxClaimLength::get();
		let claim = vec![0; d as usize];
		let caller: T::AccountId = whitelisted_caller();
		insert_claim::<T>(&claim, &caller);
	}: _(RawOrigin::Signed(caller), claim)

	transfer_claim {
		let d in 0 .. T::MaxClaimLength::get();
		let claim = vec![0; d as usize];
		let caller: T::AccountId = whitelisted_caller();
		let dest: T::AccountId = whitelisted_caller();
		insert_claim::<T>(&claim, &caller);
	}: _(RawOrigin::Signed(caller), claim, dest)

	impl_benchmark_test_suite!(PoeModule, crate::mock::new_test_ext(), crate::mock::Test);
}
