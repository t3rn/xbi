use crate::{
    _Messenger, _hrmp_channel_parachain_inherent_data, _process_messages, INITIAL_BALANCE,
};
use frame_support::traits::GenesisBuild;
use xcm_emulator::decl_test_parachain;

#[cfg(test)]
use crate::ALICE;
#[cfg(test)]
use codec::Encode;
#[cfg(test)]
use frame_support::assert_ok;
#[cfg(test)]
use xcm::prelude::*;

pub const SLIM_PARA_ID: u32 = 1;
pub const SLENDER_PARA_ID: u32 = 2;

decl_test_parachain! {
    pub struct Slim {
        Runtime = slim::Runtime,
        Origin = slim::Origin,
        XcmpMessageHandler = slim::XcmpQueue,
        DmpMessageHandler = slim::DmpQueue,
        new_ext = slim_ext(SLIM_PARA_ID),
    }
}

decl_test_parachain! {
    pub struct Slender {
        Runtime = slim::Runtime,
        Origin = slim::Origin,
        XcmpMessageHandler = slim::XcmpQueue,
        DmpMessageHandler = slim::DmpQueue,
        new_ext = slim_ext(SLENDER_PARA_ID),
    }
}

pub fn slim_ext(para_id: u32) -> sp_io::TestExternalities {
    use slim::{Runtime, System};

    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    let parachain_info_config = parachain_info::GenesisConfig {
        parachain_id: para_id.into(),
    };

    <parachain_info::GenesisConfig as GenesisBuild<Runtime, _>>::assimilate_storage(
        &parachain_info_config,
        &mut t,
    )
    .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![
            // (ALICE, INITIAL_BALANCE),
            (slim::XbiSovereign::get(), INITIAL_BALANCE),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

#[cfg(test)]
pub fn log_all_events(chain: &str) {
    slim::System::events()
        .iter()
        .for_each(|r| println!(">>> [{}] {:?}", chain, r.event));
}

#[cfg(test)]
pub fn register_asset(id: u32, location: MultiLocation, which: &str) {
    assert_ok!(slim::AssetRegistry::register(
        slim::Origin::root(),
        location.clone(),
        id
    ));
    assert_ok!(slim::AssetRegistry::register_info(
        slim::Origin::root(),
        pallet_asset_registry::AssetInfo::new(id, location.clone(), vec![]) // FIXME: add capabilities
    ));

    log_all_events(which);
    assert!(slim::System::events().iter().any(|r| matches!(
            &r.event,
            slim::Event::AssetRegistry(pallet_asset_registry::Event::Registered { asset_id, location: loc }) if asset_id == &id && &location == loc
        )));
    assert!(slim::System::events().iter().any(|r| matches!(
            &r.event,
            slim::Event::AssetRegistry(pallet_asset_registry::Event::Info { asset_id, location: loc }) if asset_id == &id && &location == loc
        )));
    slim::System::reset_events();
}

#[cfg(test)]
pub fn create_asset(
    id: u32,
    name: &str,
    symbol: &str,
    decimals: u8,
    owner: Option<sp_runtime::AccountId32>,
    min_balance: u128,
    which: &str,
) {
    assert_ok!(slim::Assets::force_create(
        slim::Origin::root(),
        id,
        owner.unwrap_or(ALICE),
        true,
        min_balance
    ));
    assert_ok!(slim::Assets::set_metadata(
        slim::Origin::signed(ALICE),
        id,
        name.encode(),
        symbol.encode(),
        decimals
    ));
    log_all_events(which);
    assert!(slim::System::events().iter().any(|r| matches!(
        r.event,
        slim::Event::Assets(pallet_assets::Event::ForceCreated { asset_id, .. }) if asset_id == id
    )));
    let n = name;
    let s = symbol;

    assert!(slim::System::events().iter().any(|r| matches!(
        &r.event,
        slim::Event::Assets(pallet_assets::Event::MetadataSet {
            asset_id,
            name,
            symbol,
            ..
        }) if asset_id == &id && name == &n.encode() && symbol == &s.encode()
    )));
    slim::System::reset_events();
}

#[cfg(test)]
pub fn mint_asset(id: u32, to: sp_runtime::AccountId32, amount: u128) {
    assert_ok!(slim::Assets::mint(
        slim::Origin::signed(ALICE),
        id,
        to,
        amount
    ));
    log_all_events("Slim");
    // assert!(slim::System::events().iter().any(|r| matches!(
    //     r.event,
    //     slim::Event::Assets(pallet_assets::Event::ForceCreated { asset_id, .. }) if asset_id == id
    // )));
    slim::System::reset_events();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Network, RococoNet};

    use codec::Encode;
    use frame_support::{assert_ok, traits::Currency};
    use polkadot_primitives::v2::Id as ParaId;
    use sp_runtime::traits::AccountIdConversion;
    use xcm::{VersionedMultiLocation, VersionedXcm};
    use xcm_emulator::TestExt;

    #[test]
    fn dmp() {
        Network::reset();

        let remark = slim::Call::System(frame_system::Call::<slim::Runtime>::remark_with_event {
            remark: "Hello from Rococo!".as_bytes().to_vec(),
        });
        RococoNet::execute_with(|| {
            assert_ok!(rococo_runtime::XcmPallet::force_default_xcm_version(
                rococo_runtime::Origin::root(),
                Some(0)
            ));
            assert_ok!(rococo_runtime::XcmPallet::send_xcm(
                Here,
                Parachain(1),
                Xcm(vec![Transact {
                    origin_type: OriginKind::SovereignAccount,
                    require_weight_at_most: INITIAL_BALANCE as u64,
                    call: remark.encode().into(),
                }]),
            ));
        });

        Slim::execute_with(|| {
            use slim::{Event, System};
            log_all_events("Slim");

            assert!(System::events().iter().any(|r| matches!(
                r.event,
                Event::System(frame_system::Event::Remarked { sender: _, hash: _ })
            )));
        });
    }

    #[test]
    fn ump() {
        Network::reset();

        RococoNet::execute_with(|| {
            let _ = rococo_runtime::Balances::deposit_creating(
                &ParaId::from(1).into_account_truncating(),
                1_000_000_000_000,
            );
        });

        let remark = rococo_runtime::Call::System(
            frame_system::Call::<rococo_runtime::Runtime>::remark_with_event {
                remark: "Hello from Pumpkin!".as_bytes().to_vec(),
            },
        );
        Slim::execute_with(|| {
            assert_ok!(slim::PolkadotXcm::send_xcm(
                Here,
                Parent,
                Xcm(vec![Transact {
                    origin_type: OriginKind::SovereignAccount,
                    require_weight_at_most: INITIAL_BALANCE as u64,
                    call: remark.encode().into(),
                }]),
            ));
        });

        RococoNet::execute_with(|| {
            use rococo_runtime::{Event, System};

            System::events()
                .iter()
                .for_each(|r| println!(">>> [RelayChain] {:?}", r.event));

            assert!(System::events().iter().any(|r| matches!(
                r.event,
                Event::System(frame_system::Event::NewAccount { account: _ })
            )));
        });
    }

    #[test]
    fn xcmp() {
        Network::reset();

        let remark = slim::Call::System(frame_system::Call::<slim::Runtime>::remark_with_event {
            remark: "Hello from Pumpkin!".as_bytes().to_vec(),
        });

        Slim::execute_with(|| {
            assert_ok!(slim::PolkadotXcm::send_xcm(
                Here,
                MultiLocation::new(1, X1(Parachain(2))),
                Xcm(vec![Transact {
                    origin_type: OriginKind::SovereignAccount,
                    require_weight_at_most: 10_000_000,
                    call: remark.encode().into(),
                }]),
            ));

            log_all_events("Slim");
        });

        Slender::execute_with(|| {
            use slim::{Event, System};
            log_all_events("Slender");

            assert!(System::events().iter().any(|r| matches!(
                r.event,
                Event::System(frame_system::Event::Remarked { sender: _, hash: _ })
            )));
        });
    }

    #[test]
    fn xcmp_through_a_parachain() {
        Network::reset();

        use slim::{Call, PolkadotXcm, Runtime};

        // The message goes through: Pumpkin --> Mushroom --> Octopus
        let remark = Call::System(frame_system::Call::<Runtime>::remark_with_event {
            remark: "Hello from Pumpkin!".as_bytes().to_vec(),
        });

        let send_xcm_to_t1rn = Call::PolkadotXcm(pallet_xcm::Call::<Runtime>::send {
            dest: Box::new(VersionedMultiLocation::V1(MultiLocation::new(
                1,
                X1(Parachain(SLENDER_PARA_ID)),
            ))),
            message: Box::new(VersionedXcm::V2(Xcm(vec![Transact {
                origin_type: OriginKind::SovereignAccount,
                require_weight_at_most: 10_000_000,
                call: remark.encode().into(),
            }]))),
        });
        Slim::execute_with(|| {
            assert_ok!(PolkadotXcm::send_xcm(
                Here,
                MultiLocation::new(1, X1(Parachain(SLIM_PARA_ID))),
                Xcm(vec![Transact {
                    origin_type: OriginKind::SovereignAccount,
                    require_weight_at_most: 100_000_000,
                    call: send_xcm_to_t1rn.encode().into(),
                }]),
            ));
        });

        Slim::execute_with(|| {
            use slim::{Event, System};
            log_all_events("Slim");

            assert!(System::events().iter().any(|r| matches!(
                r.event,
                Event::XcmpQueue(cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. })
            )));
            assert!(System::events().iter().any(|r| matches!(
                r.event,
                Event::PolkadotXcm(pallet_xcm::Event::Sent(_, _, _))
            )));
        });

        Slender::execute_with(|| {
            use slim::{Event, System};
            // execution would fail, but good enough to check if the message is received
            log_all_events("Slender");

            assert!(System::events().iter().any(|r| matches!(
                r.event,
                Event::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail { .. })
            )));
        });
    }

    #[test]
    fn deduplicate_dmp() {
        Network::reset();
        RococoNet::execute_with(|| {
            assert_ok!(rococo_runtime::XcmPallet::force_default_xcm_version(
                rococo_runtime::Origin::root(),
                Some(0)
            ));
        });

        rococo_send_rmrk("Rococo", 2);
        parachain_receive_and_reset_events(true);

        // a different dmp message in same relay-parent-block allow execution.
        rococo_send_rmrk("Polkadot", 1);
        parachain_receive_and_reset_events(true);

        // same dmp message with same relay-parent-block wouldn't execution
        rococo_send_rmrk("Rococo", 1);
        parachain_receive_and_reset_events(false);

        // different relay-parent-block allow dmp message execution
        RococoNet::execute_with(|| rococo_runtime::System::set_block_number(2));

        rococo_send_rmrk("Rococo", 1);
        parachain_receive_and_reset_events(true);

        // reset can send same dmp message again
        Network::reset();
        RococoNet::execute_with(|| {
            assert_ok!(rococo_runtime::XcmPallet::force_default_xcm_version(
                rococo_runtime::Origin::root(),
                Some(0)
            ));
        });

        rococo_send_rmrk("Rococo", 1);
        parachain_receive_and_reset_events(true);
    }

    fn rococo_send_rmrk(msg: &str, count: u32) {
        let remark = slim::Call::System(frame_system::Call::<slim::Runtime>::remark_with_event {
            remark: msg.as_bytes().to_vec(),
        });
        RococoNet::execute_with(|| {
            for _ in 0..count {
                assert_ok!(rococo_runtime::XcmPallet::send_xcm(
                    Here,
                    Parachain(1),
                    Xcm(vec![Transact {
                        origin_type: OriginKind::SovereignAccount,
                        require_weight_at_most: INITIAL_BALANCE as u64,
                        call: remark.encode().into(),
                    }]),
                ));
            }
        });
    }

    fn parachain_receive_and_reset_events(received: bool) {
        Slim::execute_with(|| {
            use slim::{Event, System};
            System::events()
                .iter()
                .for_each(|r| println!(">>> {:?}", r.event));

            if received {
                assert!(System::events().iter().any(|r| matches!(
                    r.event,
                    Event::System(frame_system::Event::Remarked { sender: _, hash: _ })
                )));

                System::reset_events();
            } else {
                assert!(System::events().iter().all(|r| !matches!(
                    r.event,
                    Event::System(frame_system::Event::Remarked { sender: _, hash: _ })
                )));
            }
        });
    }
}
