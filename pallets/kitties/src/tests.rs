use crate::{Error, Event, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

//测试创建kitty 
#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		System::assert_has_event(mock::Event::KittiesModule(Event::KittyCreated(1, 0)));
	});
}

#[test]
fn create_failed_when_index_max() {
	new_test_ext().execute_with(|| {
		KittiesCount::<Test>::put(u32::max_value());
		assert_noop!(
			KittiesModule::create(Origin::signed(1)),
			Error::<Test>::KittiesCountOverflow
		);
	})
}

#[test]
fn create_not_enough_balance() {
	new_test_ext().execute_with(|| {
		assert_noop!(KittiesModule::create(Origin::signed(3)), Error::<Test>::NotEnoughBalance);
	})
}

#[test]
fn transfer_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::transfer(Origin::signed(1), 2, 0));
	})
}

#[test]
fn transfer_failed_not_owner() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(KittiesModule::transfer(Origin::signed(2), 3, 0), Error::<Test>::NotOwner);
	})
}

#[test]
fn breed_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::breed(Origin::signed(1), 0, 1));
	})
}

#[test]
fn breed_failed() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(KittiesModule::breed(Origin::signed(1), 0, 0), Error::<Test>::SameParentIndex);
	})
}

#[test]
fn buy_kitty_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::sell(Origin::signed(1), 0, Some(100)));

		assert_ok!(KittiesModule::buy(Origin::signed(2), 0));
		assert_eq!(KittiesPrice::<Test>::contains_key(0), false);
	})
}

#[test]
fn buy_kitty_failed() {
	new_test_ext().execute_with(|| {
		//测试kitty id 无效
		assert_noop!(KittiesModule::buy(Origin::signed(1), 0), Error::<Test>::InvalidKittyIndex);
		//测试owner
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// assert_noop!(KittiesModule::buy(Origin::signed(1), 0), Error::<Test>::FromSameTo);
		//测试没有可购买时
		assert_noop!(KittiesModule::buy(Origin::signed(2), 0), Error::<Test>::NotForSale);
		//测试没有balance
		assert_ok!(KittiesModule::sell(Origin::signed(1), 0, Some(100)));
		assert_noop!(KittiesModule::buy(Origin::signed(3), 0), Error::<Test>::NotEnoughBalance);
	})
}

#[test]
fn sell_kitty_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::sell(Origin::signed(1), 0, Some(100)));
	})
}

// #[test]
// fn sell_kitty_failed() {
// 	new_test_ext().execute_with(|| {
// 		assert_noop!(KittiesModule::sell(Origin::signed(1), 1, Some(100)), Error::<Test>::FromSameTo);
// 	})
// }