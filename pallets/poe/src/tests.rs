use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};

// 【创建存证的测试用例】
#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![1, 2, 3, 4];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((1u64, frame_system::Pallet::<Test>::block_number()))
		);
	})
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![1, 2, 3, 4];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyExist
		);
	})
}

// 【撤销存证的测试用例】
// 成功撤销存证
#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![1, 2, 3, 4];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		assert_eq!(Proofs::<Test>::get(&bounded_claim), None);
	})
}

#[test]
// 撤销存证不存在
fn revoke_claim_failed_when_claim_is_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![1, 2, 3, 4];

		// 返回错误，对链上状态不进行任何修改
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimNotExist
		);
	})
}

// 撤销存证不是本人
#[test]
fn revoke_claim_failed_when_not_claim_owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![1, 2, 3, 4];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(2), claim.clone()),
			Error::<Test>::NotClaimOwner
		);
	})
}

// 【转移存证的测试用例】
// 成功转移存证
#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![1, 2, 3, 4];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2));

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((2u64, frame_system::Pallet::<Test>::block_number()))
		);
	})
}

// 转移存证失败=>存证不存在
#[test]
fn transfer_claim_failed_when_claim_is_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2),
			Error::<Test>::ClaimNotExist
		);
	})
}

// 转移存证失败=>存证非所有者
#[test]
fn transfer_claim_failed_with_wrong_owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(2), claim.clone(), 3),
			Error::<Test>::NotClaimOwner
		);
	})
}
