use crate::{
    mock::*, AssetInfo, BalanceOf, Capability, Error, ShouldExecute, WeightAssetConvert,
    WeightTrader,
};
use frame_support::{
    assert_err, assert_ok,
    traits::ProcessMessageError,
    weights::{IdentityFee, Weight},
};
use sp_runtime::{DispatchError, Either};
use std::collections::BTreeMap;
use xcm::prelude::{AssetId, *};
use xcm_executor::{traits::Properties, Assets};
fn default_registration() {
    assert_ok!(AssetRegistry::register(
        RuntimeOrigin::signed(1),
        MultiLocation {
            parents: 0,
            interior: Junctions::X1(Junction::AccountId32 {
                network: Some(NetworkId::Polkadot),
                id: [5_u8; 32]
            })
        },
        1
    ));
}

fn default_register_info() -> (u32, MultiLocation) {
    let location = MultiLocation {
        parents: 0,
        interior: Junctions::X1(Junction::AccountId32 {
            network: Some(NetworkId::Polkadot),
            id: [5_u8; 32],
        }),
    };

    let capabilities = vec![
        Capability::Reserve(Some(1u64)),
        Capability::Teleport(Some(1u64)),
        Capability::Payable {
            fees_per_weight: Some(1u128),
        },
    ];

    let id: u32 = 1;

    assert_ok!(AssetRegistry::register_info(
        RuntimeOrigin::root(),
        AssetInfo {
            id,
            capabilities,
            location: location.clone()
        }
    ));

    (id, location)
}

#[test]
fn cant_register_relay() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_err!(
            AssetRegistry::register(RuntimeOrigin::signed(1), MultiLocation::parent(), 1),
            Error::<Test>::LocationUnallowed
        );
    });
}

#[test]
fn cant_register_self() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_err!(
            AssetRegistry::register(RuntimeOrigin::signed(1), MultiLocation::here(), 1),
            Error::<Test>::LocationUnallowed
        );
    });
}
#[test]
fn cant_register_parachain() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_err!(
            AssetRegistry::register(
                RuntimeOrigin::signed(1),
                MultiLocation {
                    parents: 0,
                    interior: Junctions::X1(Junction::Parachain(1))
                },
                1
            ),
            Error::<Test>::LocationUnallowed
        );
    });
}

#[test]
fn cant_register_parachain_ever() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_err!(
            AssetRegistry::register(
                RuntimeOrigin::signed(1),
                MultiLocation {
                    parents: 0,
                    interior: Junctions::X3(
                        Junction::PalletInstance(50),
                        Junction::AccountId32 {
                            network: Some(NetworkId::Polkadot),
                            id: [5_u8; 32]
                        },
                        Junction::Parachain(1)
                    )
                },
                1
            ),
            Error::<Test>::LocationUnallowed
        );
    });
}

#[test]
fn cant_put_duplicate_capabilities() {
    new_test_ext().execute_with(|| {
        default_registration();

        assert_err!(
            AssetRegistry::register_info(
                RuntimeOrigin::root(),
                AssetInfo {
                    id: 1,
                    capabilities: vec![
                        Capability::Reserve(Some(1u64)),
                        Capability::Reserve(Some(1u64))
                    ],
                    location: MultiLocation {
                        parents: 0,
                        interior: Junctions::X1(Junction::AccountId32 {
                            network: Some(NetworkId::Polkadot),
                            id: [5_u8; 32]
                        })
                    },
                },
            ),
            Error::<Test>::CapabilitiesNotPermitted
        );

        assert_eq!(AssetRegistry::asset_metadata(1), None);
    });
}

#[test]
fn cant_put_capabilities_as_non_root() {
    new_test_ext().execute_with(|| {
        default_registration();

        assert_err!(
            AssetRegistry::register_info(
                RuntimeOrigin::signed(2),
                AssetInfo {
                    id: 1,
                    capabilities: vec![
                        Capability::Reserve(Some(1u64)),
                        Capability::Reserve(Some(1u64))
                    ],
                    location: MultiLocation {
                        parents: 0,
                        interior: Junctions::X1(Junction::AccountId32 {
                            network: Some(NetworkId::Polkadot),
                            id: [5_u8; 32]
                        })
                    },
                },
            ),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn cant_check_capability_for_unknown_asset_id() {
    new_test_ext().execute_with(|| {
        default_registration();

        assert_err!(
            crate::pallet::check_capabilities::<Test>(
                Either::Left(2),
                vec![Capability::Teleport(None)]
            ),
            Error::<Test>::NotFound
        );
    });
}

#[test]
fn cant_check_capability_for_unknown_location() {
    new_test_ext().execute_with(|| {
        default_registration();

        assert_err!(
            crate::pallet::check_capabilities::<Test>(
                Either::Right(MultiLocation {
                    parents: 0,
                    interior: Junctions::X1(Junction::AccountId32 {
                        network: Some(NetworkId::Polkadot),
                        id: [2_u8; 32]
                    })
                },),
                vec![Capability::Teleport(None)]
            ),
            Error::<Test>::NotFound
        );
    });
}

#[test]
fn cant_check_capability_for_unknown_capability() {
    new_test_ext().execute_with(|| {
        default_registration();
        let (_, _) = default_register_info();

        assert_err!(
            crate::pallet::check_capabilities::<Test>(
                Either::Left(1),
                vec![
                    Capability::Teleport(Some(2u64)),
                    Capability::Reserve(Some(1u64))
                ]
            ),
            Error::<Test>::CapabilitiesNotPermitted
        );

        assert_err!(
            crate::pallet::check_capabilities::<Test>(
                Either::Left(1),
                vec![
                    Capability::Teleport(Some(1u64)),
                    Capability::Reserve(Some(2u64))
                ]
            ),
            Error::<Test>::CapabilitiesNotPermitted
        );

        assert_err!(
            crate::pallet::check_capabilities::<Test>(
                Either::Left(1),
                vec![Capability::Payable {
                    fees_per_weight: Some(2u128)
                }]
            ),
            Error::<Test>::CapabilitiesNotPermitted
        );
    });
}

#[test]
fn can_put_new_capabilities() {
    new_test_ext().execute_with(|| {
        default_registration();

        assert_eq!(AssetRegistry::asset_metadata(1), None);
        assert_ok!(AssetRegistry::register_info(
            RuntimeOrigin::root(),
            AssetInfo {
                id: 1,
                capabilities: vec![Capability::Reserve(Some(1u64))],
                location: MultiLocation {
                    parents: 0,
                    interior: Junctions::X1(Junction::AccountId32 {
                        network: Some(NetworkId::Polkadot),
                        id: [5_u8; 32]
                    })
                },
            },
        ));

        assert_eq!(
            AssetRegistry::asset_metadata(1),
            Some(crate::AssetInfo {
                id: 1,
                capabilities: vec![Capability::Reserve(Some(1u64))],
                location: MultiLocation {
                    parents: 0,
                    interior: Junctions::X1(Junction::AccountId32 {
                        network: Some(NetworkId::Polkadot),
                        id: [5_u8; 32]
                    })
                },
            })
        );
    });
}

#[test]
fn can_overwrite_capabilities() {
    new_test_ext().execute_with(|| {
        default_registration();
        let (_, _) = default_register_info();
        assert_eq!(
            AssetRegistry::asset_metadata(1),
            Some(crate::AssetInfo {
                id: 1,
                capabilities: vec![
                    Capability::Reserve(Some(1u64)),
                    Capability::Teleport(Some(1u64)),
                    Capability::Payable {
                        fees_per_weight: Some(1u128)
                    }
                ],
                location: MultiLocation {
                    parents: 0,
                    interior: Junctions::X1(Junction::AccountId32 {
                        network: Some(NetworkId::Polkadot),
                        id: [5_u8; 32]
                    })
                },
            })
        );

        assert_ok!(AssetRegistry::register_info(
            RuntimeOrigin::root(),
            AssetInfo {
                id: 1,
                capabilities: vec![Capability::Reserve(Some(1u64))],
                location: MultiLocation {
                    parents: 0,
                    interior: Junctions::X1(Junction::AccountId32 {
                        network: Some(NetworkId::Polkadot),
                        id: [5_u8; 32]
                    })
                },
            },
        ));

        assert_eq!(
            AssetRegistry::asset_metadata(1),
            Some(crate::AssetInfo {
                id: 1,
                capabilities: vec![Capability::Reserve(Some(1u64))],
                location: MultiLocation {
                    parents: 0,
                    interior: Junctions::X1(Junction::AccountId32 {
                        network: Some(NetworkId::Polkadot),
                        id: [5_u8; 32]
                    })
                },
            })
        );
    });
}

#[test]
fn can_check_available_capabilities_with_asset_id() {
    new_test_ext().execute_with(|| {
        default_registration();
        let (_, _) = default_register_info();

        assert_eq!(
            crate::pallet::check_capabilities::<Test>(
                Either::Left(1),
                vec![Capability::Reserve(Some(1u64))]
            ),
            Ok(vec![Capability::Reserve(Some(1u64))])
        );
    });
}

#[test]
fn can_check_available_capabilities_with_location() {
    new_test_ext().execute_with(|| {
        default_registration();
        let (_, _) = default_register_info();

        assert_eq!(
            crate::pallet::check_capabilities::<Test>(
                Either::Right(MultiLocation {
                    parents: 0,
                    interior: Junctions::X1(Junction::AccountId32 {
                        network: Some(NetworkId::Polkadot),
                        id: [5_u8; 32]
                    })
                },),
                vec![Capability::Reserve(1u64.into())]
            ),
            Ok(vec![Capability::Reserve(Some(1u64))])
        );
    });
}

#[test]
fn can_check_soft_matches() {
    new_test_ext().execute_with(|| {
        default_registration();
        let (_, _) = default_register_info();

        assert_eq!(
            crate::pallet::check_capabilities::<Test>(
                Either::Left(1),
                vec![
                    Capability::Reserve(None),
                    Capability::Teleport(None),
                    Capability::Payable {
                        fees_per_weight: None
                    }
                ]
            ),
            Ok(vec![
                Capability::Reserve(Some(1u64)),
                Capability::Teleport(Some(1u64)),
                Capability::Payable {
                    fees_per_weight: Some(1u128)
                }
            ])
        );
    })
}

#[test]
fn can_check_exact_matches() {
    new_test_ext().execute_with(|| {
        default_registration();
        let (_, _) = default_register_info();

        assert_eq!(
            crate::pallet::check_capabilities::<Test>(
                Either::Left(1),
                vec![
                    Capability::Reserve(Some(1u64)),
                    Capability::Teleport(Some(1u64)),
                    Capability::Payable {
                        fees_per_weight: Some(1u128)
                    }
                ]
            ),
            Ok(vec![
                Capability::Reserve(Some(1u64)),
                Capability::Teleport(Some(1u64)),
                Capability::Payable {
                    fees_per_weight: Some(1u128)
                }
            ])
        );
    })
}

#[test]
fn can_check_mixed_matches() {
    new_test_ext().execute_with(|| {
        default_registration();
        let (_, _) = default_register_info();

        assert_eq!(
            crate::pallet::check_capabilities::<Test>(
                Either::Left(1),
                vec![
                    Capability::Reserve(Some(1u64)),
                    Capability::Teleport(None),
                    Capability::Payable {
                        fees_per_weight: None
                    }
                ]
            ),
            Ok(vec![
                Capability::Reserve(Some(1u64)),
                Capability::Teleport(Some(1u64)),
                Capability::Payable {
                    fees_per_weight: Some(1u128)
                }
            ])
        );
    })
}

#[test]
fn xcm_message_capabilities_are_generated_correctly() {
    new_test_ext().execute_with(|| {
        default_registration();
        let (_, _) = default_register_info();

        assert_eq!(
            Capability::<<Test as frame_system::Config>::AccountId, BalanceOf<Test>>::try_from(
                &WithdrawAsset::<()>((Here, 1).into())
            ),
            Ok(Capability::Payable {
                fees_per_weight: None
            })
        );
    })
}

#[test]
fn can_execute_xcm_message_with_matching_capabilities() {
    new_test_ext().execute_with(|| {
        default_registration();
        let (_, location) = default_register_info();

        let mut msg = vec![
            WithdrawAsset((Here, 1).into()),
            WithdrawAsset((Here, 1).into()),
        ];

        assert_ok!(crate::Pallet::<Test>::should_execute::<()>(
            &location,
            &mut msg,
            Default::default(),
            &mut Properties {
                weight_credit: Weight::zero(),
                message_id: None
            },
        ));
    })
}

#[test]
fn cant_execute_xcm_message_with_wrong_capabilities() {
    new_test_ext().execute_with(|| {
        default_registration();
        let location = MultiLocation {
            parents: 0,
            interior: Junctions::X1(Junction::AccountId32 {
                network: Some(NetworkId::Polkadot),
                id: [5_u8; 32],
            }),
        };

        assert_ok!(AssetRegistry::register_info(
            RuntimeOrigin::root(),
            AssetInfo {
                id: 1,
                capabilities: vec![Capability::Reserve(Some(1u64))],
                location: location.clone(),
            }
        ));

        let mut msg = vec![WithdrawAsset((Here, 1).into())];

        let weight = BaseXcmWeight::get();

        assert_err!(
            crate::Pallet::<Test>::should_execute::<()>(
                &location,
                &mut msg,
                Default::default(),
                &mut Properties {
                    weight_credit: Weight::zero(),
                    message_id: None
                },
            ),
            ProcessMessageError::BadFormat
        )
    })
}

#[test]
fn can_buy_and_refund_weight_multiple_assets() {
    new_test_ext().execute_with(|| {
        default_registration();
        let (_, location_one) = default_register_info();

        let location_two = MultiLocation {
            parents: 0,
            interior: Junctions::X1(Junction::AccountId32 {
                network: Some(NetworkId::Kusama),
                id: [4_u8; 32],
            }),
        };

        let capabilities = vec![
            Capability::Reserve(Some(1u64)),
            Capability::Teleport(Some(1u64)),
        ];

        let id: u32 = 2;

        assert_ok!(AssetRegistry::register_info(
            RuntimeOrigin::root(),
            AssetInfo {
                id,
                capabilities,
                location: location_two.clone()
            }
        ));

        let mut assets: Assets = vec![
            (Concrete(location_one.clone()), 4_000_000u128).into(),
            (Concrete(location_two.clone()), 4_000_000u128).into(),
        ]
        .into();

        let mut trader = WeightAssetConvert::<Test, IdentityFee<BalanceOf<Test>>>::new();

        let mut balance: BTreeMap<AssetId, u128> = BTreeMap::new();
        balance.insert(location_one.clone().into(), 2_000_000u128);
        balance.insert(location_two.into(), 4_000_000u128);

        assets = trader
            .buy_weight(Weight::from_parts(2_000_000u64, 0u64), assets.clone())
            .unwrap();
        assert_eq!(
            assets,
            Assets {
                fungible: balance.clone(),
                non_fungible: Default::default()
            }
        );

        assert_eq!(
            trader
                .refund_weight(Weight::from_parts(2_000_000u64, 0u64))
                .unwrap(),
            (Concrete(location_one), 2_000_000u128).into()
        );

        assert_eq!(trader.weight, Default::default());
        assert_eq!(trader.refund_weight(Weight::from_parts(1u64, 0u64)), None);
    })
}

#[test]
fn can_buy_weight_for_partial_balance() {
    new_test_ext().execute_with(|| {
        default_registration();
        let (_, location) = default_register_info();

        // We are going to buy 4e9 weight
        let weight_to_buy = Weight::from_parts(2_000_000u64, 0u64);
        let asset: MultiAsset = (Concrete(location.clone()), 4_000_000u128).into();

        let mut trader = WeightAssetConvert::<Test, IdentityFee<BalanceOf<Test>>>::new();

        // Generate expected fungible balance
        let mut balance: BTreeMap<AssetId, u128> = BTreeMap::new();
        balance.insert(location.clone().into(), 2_000_000u128);

        assert_eq!(
            trader.buy_weight(weight_to_buy, asset.into()),
            Ok(Assets {
                fungible: balance.clone(),
                non_fungible: Default::default()
            })
        );

        // can refund in multiple steps
        assert_eq!(
            trader.refund_weight(Weight::from_parts(1_000_000u64, 0u64)),
            Some((Concrete(location.clone()), 1_000_000u128).into())
        );

        assert_eq!(
            trader.refund_weight(Weight::from_parts(1_000_000u64, 0u64)),
            Some((Concrete(location), 1_000_000u128).into())
        );

        // Weight has been deducted correctly
        assert_eq!(trader.weight, Default::default());
        assert_eq!(trader.refund_weight(Weight::from_parts(1u64, 0u64)), None);
    })
}

#[test]
fn can_buy_and_refund_weight_with_fee_weight_multiplier() {
    new_test_ext().execute_with(|| {
        default_registration();
        let location = MultiLocation {
            parents: 0,
            interior: Junctions::X1(Junction::AccountId32 {
                network: Some(NetworkId::Polkadot),
                id: [5_u8; 32],
            }),
        };

        let capabilities = vec![
            Capability::Reserve(Some(1u64)),
            Capability::Teleport(Some(1u64)),
            Capability::Payable {
                fees_per_weight: Some(3),
            },
        ];

        let id: u32 = 1;

        assert_ok!(AssetRegistry::register_info(
            RuntimeOrigin::root(),
            AssetInfo {
                id,
                capabilities,
                location: location.clone()
            }
        ));

        let weight_to_buy = Weight::from_parts(1_000_000u64, 0u64);
        let mut assets: Assets = vec![(Concrete(location.clone()), 6_000_000u128).into()].into();

        let mut trader = WeightAssetConvert::<Test, IdentityFee<BalanceOf<Test>>>::new();

        // Generate expected fungible balance
        let mut balance: BTreeMap<AssetId, u128> = BTreeMap::new();
        balance.insert(location.clone().into(), 3_000_000u128);

        // buy first half
        assets = trader.buy_weight(weight_to_buy, assets.clone()).unwrap();
        assert_eq!(
            assets,
            Assets {
                fungible: balance.clone(),
                non_fungible: Default::default()
            }
        );
        assert_eq!(trader.weight, Weight::from_parts(1_000_000u64, 0u64));

        // buy second half
        assets = trader.buy_weight(weight_to_buy, assets.clone()).unwrap();
        assert_eq!(
            assets,
            Assets {
                fungible: Default::default(), // these are empty
                non_fungible: Default::default()
            }
        );
        assert_eq!(trader.weight, Weight::from_parts(2_000_000u64, 0u64));

        assert_eq!(
            trader.refund_weight(Weight::from_parts(2_000_000u64, 0u64)),
            Some((Concrete(location), 6_000_000u128).into())
        );

        assert_eq!(trader.weight, Default::default());
        assert_eq!(trader.refund_weight(weight_to_buy), None);
    })
}

#[test]
fn can_buy_and_refund_weight_for_whole_balance() {
    new_test_ext().execute_with(|| {
        default_registration();
        let (_, location) = default_register_info();

        // We are going to buy 4e9 weight
        let bought = Weight::from_parts(4_000_000u64, 0u64);
        let asset: MultiAsset = MultiAsset {
            id: Concrete(location),
            fun: Fungibility::Fungible(bought.ref_time() as u128),
        };

        let mut trader = WeightAssetConvert::<Test, IdentityFee<BalanceOf<Test>>>::new();

        assert_eq!(
            trader.buy_weight(bought, asset.clone().into()),
            Ok(Assets {
                fungible: Default::default(),
                non_fungible: Default::default()
            })
        );

        assert_eq!(trader.refund_weight(bought), Some(asset));

        // Weight has been deducted correctly
        assert_eq!(trader.weight, Default::default());
        assert_eq!(trader.refund_weight(Weight::from_parts(1, 0u64)), None);
    })
}

#[test]
fn cant_buy_weight_for_insufficient_balance() {
    new_test_ext().execute_with(|| {
        default_registration();
        let (_, location) = default_register_info();

        let weight_to_buy = Weight::from_parts(2_000_000u64, 0u64);
        let asset: MultiAsset = (Concrete(location), 1_000_000u128).into();

        let mut trader = WeightAssetConvert::<Test, IdentityFee<BalanceOf<Test>>>::new();

        assert_err!(
            trader.buy_weight(weight_to_buy, asset.into()),
            XcmError::TooExpensive
        );

        assert_eq!(trader.weight, Default::default());
        assert_eq!(trader.refund_weight(weight_to_buy), None);
    })
}

#[test]
fn cant_buy_weight_without_asset() {
    new_test_ext().execute_with(|| {
        default_registration();

        let weight_to_buy = Weight::from_parts(1_000_000u64, 0u64);
        let mut trader = WeightAssetConvert::<Test, IdentityFee<BalanceOf<Test>>>::new();

        assert_err!(
            trader.buy_weight(weight_to_buy, vec![].into()),
            XcmError::AssetNotFound
        );
    })
}

#[test]
fn cant_buy_weight_with_abstract_asset() {
    new_test_ext().execute_with(|| {
        default_registration();
        let (_, _) = default_register_info();

        // We are going to buy 4e9 weight
        let weight_to_buy = Weight::from_parts(1_000_000u64, 0u64);
        let asset: MultiAsset = (Abstract(Default::default()), 1_000_000u128).into();

        let mut trader = WeightAssetConvert::<Test, IdentityFee<BalanceOf<Test>>>::new();

        assert_err!(
            trader.buy_weight(weight_to_buy, asset.into()),
            XcmError::AssetNotFound
        );
    })
}

#[test]
fn cant_buy_weight_with_unknown_asset() {
    new_test_ext().execute_with(|| {
        default_registration();
        let location = MultiLocation {
            parents: 0,
            interior: Junctions::X1(Junction::AccountId32 {
                network: Some(NetworkId::Polkadot),
                id: [5_u8; 32],
            }),
        };

        // We are going to buy 4e9 weight
        let weight_to_buy = Weight::from_parts(1_000_000u64, 0u64);
        let asset: MultiAsset = (Concrete(location), 1_000_000u128).into();
        let mut trader = WeightAssetConvert::<Test, IdentityFee<BalanceOf<Test>>>::new();

        assert_err!(
            trader.buy_weight(weight_to_buy, asset.into()),
            XcmError::AssetNotFound
        );

        assert_eq!(trader.weight, Default::default());
        assert_eq!(trader.refund_weight(weight_to_buy), None);
    })
}

#[test]
fn cant_buy_weight_without_payable_capability() {
    new_test_ext().execute_with(|| {
        default_registration();
        let location = MultiLocation {
            parents: 0,
            interior: Junctions::X1(Junction::AccountId32 {
                network: Some(NetworkId::Polkadot),
                id: [5_u8; 32],
            }),
        };

        let capabilities = vec![
            Capability::Reserve(Some(1u64)),
            Capability::Teleport(Some(1u64)),
        ];

        let id: u32 = 1;

        assert_ok!(AssetRegistry::register_info(
            RuntimeOrigin::root(),
            AssetInfo {
                id,
                capabilities,
                location: location.clone()
            }
        ));

        // We are going to buy 4e9 weight
        let weight_to_buy = Weight::from_parts(2_000_000u64, 0u64);
        let asset: MultiAsset = (Concrete(location), 1_000_000u128).into();

        let mut trader = WeightAssetConvert::<Test, IdentityFee<BalanceOf<Test>>>::new();

        assert_err!(
            trader.buy_weight(weight_to_buy, asset.into()),
            XcmError::WeightNotComputable
        );

        assert_eq!(trader.weight, Default::default());
        assert_eq!(trader.refund_weight(weight_to_buy), None);
    })
}

#[test]
fn cant_buy_weight_with_non_payable_capability() {
    new_test_ext().execute_with(|| {
        default_registration();
        let location = MultiLocation {
            parents: 0,
            interior: Junctions::X1(Junction::AccountId32 {
                network: Some(NetworkId::Polkadot),
                id: [5_u8; 32],
            }),
        };

        let capabilities = vec![
            Capability::Reserve(Some(1u64)),
            Capability::Teleport(Some(1u64)),
            Capability::Payable {
                fees_per_weight: None,
            },
        ];

        let id: u32 = 1;

        assert_ok!(AssetRegistry::register_info(
            RuntimeOrigin::root(),
            AssetInfo {
                id,
                capabilities,
                location: location.clone()
            }
        ));

        // We are going to buy 4e9 weight
        let weight_to_buy = Weight::from_parts(1_000_000u64, 0u64);
        let asset: MultiAsset = (Concrete(location), 1_000_000u128).into();

        let mut trader = WeightAssetConvert::<Test, IdentityFee<BalanceOf<Test>>>::new();

        assert_err!(
            trader.buy_weight(weight_to_buy, asset.into()),
            XcmError::WeightNotComputable
        );

        assert_eq!(trader.weight, Default::default());
        assert_eq!(trader.refund_weight(weight_to_buy), None);
    })
}
