use crate::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
// use frame_support::traits::Get;
use frame_system::RawOrigin;
// use sp_std::vec;

benchmarks! {
	create_claim {
		let d in 0 .. T::MaxClaimLength::get();
		let claim = vec![0; d as usize];
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller), claim)

	impl_benchmark_test_suite!(PoeModule, crate::mock::new_test_ext(), crate::mock::Test);
}
