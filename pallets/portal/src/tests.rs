use crate::H256;
use crate::{mock::*, Error, XbiResponses};
use frame_support::{assert_err, assert_ok};
use xp_channel::traits::Writable;
use xp_channel::XbiResult;

#[test]
fn cannot_store_duplicate_responses() {
    new_test_ext().execute_with(|| {
        let result = XbiResult::default();
        let hash = H256::from_low_u64_be(1);
        assert_ok!(XbiPortal::write((hash.clone(), result.clone())));
        assert_err!(
            XbiPortal::write((hash.clone(), result.clone())),
            Error::<Test>::ResponseAlreadyStored
        );
        let stored = XbiResponses::<Test>::get(hash).unwrap();
        assert_eq!(stored, result);
    });
}
