use super::*;
use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_kitties() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert!(Kitties::<Test>::contains_key(0));
		assert_eq!(KittyOwner::<Test>::get(0), Some(1));
		assert_eq!(NextKittyId::<Test>::get(), 1);
		System::assert_has_event(mock::Event::KittiesModule(Event::KittyCreated(1, 0)));
	});
}

#[test]
fn create_failed() {
	new_test_ext().execute_with(|| {
		assert_noop!(KittiesModule::create(Origin::signed(4)), Error::<Test>::InsufficientBalance);
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::transfer(Origin::signed(1), 0, 2));
		// kitty_id: 0, owner: 2, count: 1
		assert!(Kitties::<Test>::contains_key(0));
		assert_eq!(KittyOwner::<Test>::get(0), Some(2));
		assert_eq!(NextKittyId::<Test>::get(), 1);
		System::assert_has_event(mock::Event::KittiesModule(Event::KittyTransferred(1, 2, 0)));
	})
}

#[test]
fn transfer_failed() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1), 2, 1),
			Error::<Test>::InvalidKittyId
		);
		assert_ok!(KittiesModule::create(Origin::signed(2)));
		assert_noop!(KittiesModule::transfer(Origin::signed(2), 0, 3), Error::<Test>::NotOwner);
	})
}

#[test]
fn breed_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::bread(Origin::signed(1), 0, 1));
		System::assert_has_event(mock::Event::KittiesModule(Event::KittyBread(1, 2)));

		// kitty_id: [0,1,2], owner: 1, count: 3
		assert!(Kitties::<Test>::contains_key(0));
		assert_eq!(KittyOwner::<Test>::get(0), Some(1));
		assert_eq!(KittyOwner::<Test>::get(1), Some(1));
		assert_eq!(KittyOwner::<Test>::get(2), Some(1));
		assert_eq!(NextKittyId::<Test>::get(), 3);
	})
}

#[test]
fn breed_failed() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// kitty_id: [0], owner: 1, count: 1
		assert_noop!(KittiesModule::breed(Origin::signed(1), 0, 0), Error::<Test>::SameKittyId);
		assert_noop!(KittiesModule::breed(Origin::signed(1), 0, 1), Error::<Test>::InvalidKittyId);
		assert_noop!(KittiesModule::breed(Origin::signed(1), 1, 0), Error::<Test>::InvalidKittyId);
	})
}
