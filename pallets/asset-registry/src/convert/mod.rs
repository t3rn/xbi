use xcm_primitives::frame_traits::AssetLookup;

use crate::*;

impl<T: Config> AssetLookup<AssetIdOf<T>> for Pallet<T> {}

impl<T: Config> xcm_executor::traits::Convert<MultiLocation, AssetIdOf<T>> for Pallet<T> {
    fn convert_ref(value: impl core::borrow::Borrow<MultiLocation>) -> Result<AssetIdOf<T>, ()> {
        let value = value.borrow();
        log::debug!(target: "xcm", "convert_ref: {:?}", value);
        Self::lookup(Either::Left(value.clone()))
            .map_err(|_| ())
            .and_then(|asset_id| asset_id.left().ok_or(()))
    }

    fn reverse_ref(value: impl core::borrow::Borrow<AssetIdOf<T>>) -> Result<MultiLocation, ()> {
        let value = value.borrow();
        log::debug!(target: "xcm", "reverse_ref: {:?}", value);
        Self::lookup(Either::Right(*value))
            .map_err(|_| ())
            .and_then(|location| location.right().ok_or(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::mock::{new_test_ext, AssetRegistry, Origin, Test};
    use frame_support::{assert_err, assert_ok};
    use xcm_executor::traits::Convert;

    fn store_asset_one_for_alice() -> (MultiLocation, u32) {
        let location = MultiLocation {
            parents: 0,
            interior: X1(AccountId32 {
                network: NetworkId::Polkadot,
                id: [1_u8; 32],
            }),
        };
        assert_ok!(AssetRegistry::register(
            Origin::signed(1),
            location.clone(),
            1
        ));
        assert_ok!(AssetRegistry::register_info(
            Origin::root(),
            AssetInfo::new(1, location.clone(), vec![])
        ));
        (location, 1)
    }

    #[test]
    fn can_lookup_loc_ref() {
        new_test_ext().execute_with(|| {
            let (location, _asset) = store_asset_one_for_alice();

            assert_ok!(crate::pallet::Pallet::<Test>::convert_ref(location));
        });
    }

    #[test]
    fn cant_lookup_non_existent_loc_ref() {
        new_test_ext().execute_with(|| {
            assert_err!(
                crate::pallet::Pallet::<Test>::convert_ref(MultiLocation::parent()),
                ()
            );
        });
    }

    #[test]
    fn can_lookup_asset_ref() {
        new_test_ext().execute_with(|| {
            let (_location, asset) = store_asset_one_for_alice();

            assert_ok!(crate::pallet::Pallet::<Test>::reverse_ref(asset));
        });
    }

    #[test]
    fn cant_lookup_non_existent_asset_ref() {
        new_test_ext().execute_with(|| {
            assert_err!(crate::pallet::Pallet::<Test>::reverse_ref(5555), ());
        });
    }
}
