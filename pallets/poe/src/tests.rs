use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};

///  创建存证
#[test]
fn create_claim_success() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let claim_vec = BoundedVec::try_from(claim.clone()).unwrap();
        // 创建
        assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1),claim.clone()));
        // 验证创建结果
        assert_eq!(
            Claims::<Test>::get(&claim_vec),
            Some((1, frame_system::Pallet::<Test>::block_number()))
        );
    })
}

///  创建存证 - 失败：存证已经存在
#[test]
fn create_claim_failed_when_already_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        // 创建
        assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1),claim.clone()));
        // 再次创建
        assert_noop!(
            PoeModule::create_claim(RuntimeOrigin::signed(1),claim.clone()),
            Error::<Test>::ClaimAlreadyExisted
        );
    })
}

/// 撤销存证
#[test]
fn revoke_claim_success() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        // 先创建
        assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1),claim.clone()));
        // 再撤销
        assert_ok!(PoeModule::revoke_claim(RuntimeOrigin::signed(1),claim.clone()));
    });
}

/// 撤销存证 - 失败：存证不存在
#[test]
fn revoke_claim_fail_when_not_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        // 不创建
        // 直接撤销
        assert_noop!(
            PoeModule::revoke_claim(RuntimeOrigin::signed(1),claim.clone()),
            Error::<Test>::NoSuchClaim
        );
    });
}

/// 撤销存证 - 失败：非存证持有人
#[test]
fn revoke_claim_fail_when_not_owner() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        // 先创建
        assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1),claim.clone()));
        // 换持有人撤销
        assert_noop!(
            PoeModule::revoke_claim(RuntimeOrigin::signed(2),claim.clone()),
            Error::<Test>::NotClaimOwner
        );
    });
}

/// 转移存证
#[test]
fn transfer_claim_success() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let receiver = 2u64;
        // 先创建
        assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1),claim.clone()));
        // 再转移
        assert_ok!(PoeModule::transfer_claim(
            RuntimeOrigin::signed(1),
            claim.clone(),
            receiver,
        ));
        let claim_vec = BoundedVec::try_from(claim.clone()).unwrap();
        // 验证转移结果
        assert_eq!(
            Claims::<Test>::get(&claim_vec),
            Some((receiver, frame_system::Pallet::<Test>::block_number()))
        );
    });
}

/// 转移存证 - 失败：存证不存在
#[test]
fn transfer_claim_fail_when_not_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let receiver = 2u64;

        // 不创建
        // 直接转移
        assert_noop!(
            PoeModule::transfer_claim(
                RuntimeOrigin::signed(1),
                claim.clone(),
                receiver
            ),
            Error::<Test>::NoSuchClaim
        );
    });
}

/// 转移存证 - 失败：非存证持有人
#[test]
fn transfer_claim_fail_when_not_owner() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let receiver = 3u64;

        // 先创建
        assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1),claim.clone()));
        // 更换持有人 转移
        assert_noop!(
            PoeModule::transfer_claim(
                RuntimeOrigin::signed(2),
                claim.clone(),
                receiver
            ),
            Error::<Test>::NotClaimOwner
        );
    });
}

/// 转移存证 - 失败：转移给自己
#[test]
fn transfer_claim_fail_when_to_self() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let receiver = 1u64;

        // 先创建
        assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1),claim.clone()));
        // 转移给自己
        assert_noop!(
            PoeModule::transfer_claim(
                RuntimeOrigin::signed(1),
                claim.clone(),
                receiver
            ),
            Error::<Test>::CanNotTransferToSelf
        );
    });
}