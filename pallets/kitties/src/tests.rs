use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};


const ACCOUNT_BALANCE: u128 = 100000;


///  创建Kitty
#[test]
fn create_kitty() {
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        // 预设余额
        assert_ok!(Balances::set_balance(RuntimeOrigin::root(), account_id, ACCOUNT_BALANCE, 0));

        // 验空
        assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
        // 验证kitty创建正常
        assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(account_id)));
        // 创建事件验证
        let kitty = KittiesModule::kitties(kitty_id).expect("Kitty Created");
        System::assert_last_event(Event::KittyCreated {
            who: account_id,
            kitty_id,
            kitty,
        }.into());
        // 验证余额扣减正确
        assert_eq!(Balances::free_balance(account_id), ACCOUNT_BALANCE - KittyPrice::get());
        assert_eq!(Balances::free_balance(&get_account_id()), KittyPrice::get());

        // 验证kittyId存储符合预期
        assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1);
        // 验证kitty的内容存在
        assert_eq!(KittiesModule::kitties(kitty_id).is_some(), true);
        // 验证创建的kitty所有者正确
        assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
        // 验证创建的kitty没有parents
        assert_eq!(KittiesModule::kitty_parents(kitty_id), None);
        // 验证溢出
        crate::NextKittyId::<Test>::set(crate::KittyId::MAX);
        // 验证重复
        assert_noop!(
            KittiesModule::create_kitty(RuntimeOrigin::signed(account_id)),
            Error::<Test>::InvalidKittyId
        );
    })
}

/// 繁衍 Kitty
#[test]
fn breed_kitty() {
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        // 预设余额
        assert_ok!(Balances::set_balance(RuntimeOrigin::root(), account_id, ACCOUNT_BALANCE, 0));

        // 验证俩父代相同
        assert_noop!(
            KittiesModule::breed(RuntimeOrigin::signed(account_id),kitty_id,kitty_id),
            Error::<Test>::SameKittyId
        );
        // 验证余额没有扣减
        assert_eq!(Balances::free_balance(account_id), ACCOUNT_BALANCE);

        // 验证kitty不存在
        assert_noop!(
            KittiesModule::breed(RuntimeOrigin::signed(account_id),kitty_id,kitty_id+1),
            Error::<Test>::InvalidKittyId
        );
        // 验证余额没有扣减
        assert_eq!(Balances::free_balance(account_id), ACCOUNT_BALANCE);

        // 验证创建两个kitty成功
        assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(account_id)));
        assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(account_id)));
        // 验证余额扣减正确
        assert_eq!(Balances::free_balance(account_id), ACCOUNT_BALANCE - 2 * KittyPrice::get());
        assert_eq!(Balances::free_balance(&get_account_id()), 2 * KittyPrice::get());

        // 验证kittyId存储符合预期
        assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 2);
        // 验证kitty繁衍成功
        assert_ok!(KittiesModule::breed(RuntimeOrigin::signed(account_id),
            kitty_id,
            kitty_id+1
        ));
        // 验证余额扣减正确
        assert_eq!(Balances::free_balance(account_id), ACCOUNT_BALANCE - 3 * KittyPrice::get());
        assert_eq!(Balances::free_balance(&get_account_id()), 3 * KittyPrice::get());

        let breed_kitty_id = 2;
        // 繁衍事件验证
        let breed_kitty = KittiesModule::kitties(breed_kitty_id).expect("Breed Kitty Created");
        System::assert_last_event(
            Event::KittyBred {
                who: account_id,
                kitty_id: breed_kitty_id,
                kitty: breed_kitty,
            }.into()
        );

        // 验证繁衍kitty成功后，kittyId的存储符合预期
        assert_eq!(KittiesModule::next_kitty_id(), breed_kitty_id + 1);
        // 验证繁衍的kitty的内容存在
        assert_eq!(KittiesModule::kitties(breed_kitty_id).is_some(), true);
        // 验证繁衍的kitty的owner符合预期
        assert_eq!(KittiesModule::kitty_owner(breed_kitty_id), Some(account_id));
        // 验证繁衍的kitty的parents正确
        assert_eq!(
            KittiesModule::kitty_parents(breed_kitty_id),
            Some((kitty_id, kitty_id + 1))
        );
    });
}

/// 转移 Kitty
#[test]
fn transfer_kitty() {
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        let recipient = 2;
        // 预设余额
        assert_ok!(Balances::set_balance(RuntimeOrigin::root(), account_id, ACCOUNT_BALANCE, 0));
        assert_ok!(Balances::set_balance(RuntimeOrigin::root(), recipient, ACCOUNT_BALANCE, 0));

        // 验证kitty创建正常
        assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(account_id)));
        // 验证创建的kitty所有者正确
        assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
        // 验证非持有者不能转移
        assert_noop!(KittiesModule::transfer(
                RuntimeOrigin::signed(recipient),
                recipient,
                kitty_id
            ),
            Error::<Test>::NotOwner
        );
        // 验证不能转移给自己
        assert_noop!(KittiesModule::transfer(
                RuntimeOrigin::signed(account_id),
                account_id,
                kitty_id
            ),
            Error::<Test>::CanNotTransferToSelf
        );
        // 转移成功
        assert_ok!(KittiesModule::transfer(
                RuntimeOrigin::signed(account_id),
                recipient,
                kitty_id
        ));
        // 转移事件验证
        System::assert_last_event(
            Event::KittyTransferred {
                who: account_id,
                recipient,
                kitty_id,
            }.into()
        );

        // 验证转移后owner正确
        assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(recipient));
        // 验证转移后，新owner能正常转移
        assert_ok!(KittiesModule::transfer(
                RuntimeOrigin::signed(recipient),
                account_id,
                kitty_id
        ));
        // 验证新owner转移后，owner正确
        assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
        // 转移事件验证
        System::assert_last_event(
            Event::KittyTransferred {
                who: recipient,
                recipient: account_id,
                kitty_id,
            }.into()
        );
    });
}

/// 标记 Kitty 可售
#[test]
fn sale_kitty() {
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        // 预设余额
        assert_ok!(Balances::set_balance(RuntimeOrigin::root(), account_id, ACCOUNT_BALANCE, 0));

        // 验空
        assert_noop!(
			KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id),
			Error::<Test>::InvalidKittyId
		);
        // 验证kitty创建正常
        assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(account_id)));
        // 验证余额扣减正确
        assert_eq!(Balances::free_balance(account_id), ACCOUNT_BALANCE - KittyPrice::get());
        assert_eq!(Balances::free_balance(&get_account_id()), KittyPrice::get());
        // 所有权不正确
        assert_noop!(
			KittiesModule::sale(RuntimeOrigin::signed(account_id + 1), kitty_id),
			Error::<Test>::NotOwner
		);
        // 标记成功
        assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id));
        assert!(KittiesModule::kitty_on_sale(kitty_id).is_some());
        System::assert_last_event(Event::KittyOnSale { who: account_id, kitty_id }.into());

        // 已经在售
        assert_noop!(
			KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id),
			Error::<Test>::AlreadyOnSale
		);
    })
}

/// 购买 Kitty
#[test]
fn buy_kitty() {
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        let account_id_2 = 2;
        // 预设余额
        assert_ok!(Balances::set_balance(RuntimeOrigin::root(), account_id, ACCOUNT_BALANCE, 0));
        assert_ok!(Balances::set_balance(RuntimeOrigin::root(), account_id_2, ACCOUNT_BALANCE, 0));

        // 验空
        assert_noop!(
			KittiesModule::buy(RuntimeOrigin::signed(account_id), kitty_id),
			Error::<Test>::InvalidKittyId
		);
        // 验证kitty创建正常
        assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(account_id)));
        // 验证余额扣减正确
        assert_eq!(Balances::free_balance(account_id), ACCOUNT_BALANCE - KittyPrice::get());
        assert_eq!(Balances::free_balance(&get_account_id()), KittyPrice::get());
        // 已经持有
        assert_noop!(
			KittiesModule::buy(RuntimeOrigin::signed(account_id), kitty_id),
			Error::<Test>::AlreadyOwned
		);
        // 非可售状态
        assert_noop!(
			KittiesModule::buy(RuntimeOrigin::signed(account_id_2), kitty_id),
			Error::<Test>::NotOnSale
		);
        // 标记在售成功
        assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id));
        assert!(KittiesModule::kitty_on_sale(kitty_id).is_some());
        System::assert_last_event(Event::KittyOnSale { who: account_id, kitty_id }.into());
        // 购买成功
        assert_ok!(KittiesModule::buy(RuntimeOrigin::signed(account_id_2), kitty_id));
        // 验证相关结果
        assert!(KittiesModule::kitty_on_sale(kitty_id).is_none());
        assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id_2));
        assert_eq!(Balances::free_balance(account_id), ACCOUNT_BALANCE);
        assert_eq!(Balances::free_balance(account_id_2), ACCOUNT_BALANCE - KittyPrice::get());

        System::assert_last_event(Event::KittyBought { who: account_id_2, kitty_id }.into());
    })
}