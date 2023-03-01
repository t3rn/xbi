use crate::{
    _Messenger, _hrmp_channel_parachain_inherent_data, _process_messages, INITIAL_BALANCE,
};
use frame_support::traits::GenesisBuild;
use xcm_emulator::decl_test_parachain;

pub const LARGE_PARA_ID: u32 = 3;

decl_test_parachain! {
    pub struct Large {
        Runtime = large::Runtime,
        Origin = large::Origin,
        XcmpMessageHandler = large::XcmpQueue,
        DmpMessageHandler = large::DmpQueue,
        new_ext = large_ext(LARGE_PARA_ID),
    }
}

pub fn large_ext(para_id: u32) -> sp_io::TestExternalities {
    use large::{Runtime, System};

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
            (large::xcm_config::XbiSovereign::get(), INITIAL_BALANCE),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        assert_deposit, assert_polkadot_attempted, assert_polkadot_sent,
        assert_relay_executed_downward, assert_relay_executed_upward, assert_withdrawal,
        assert_xbi_instruction_handled, assert_xbi_received, assert_xbi_request_handled,
        assert_xbi_sent, assert_xcmp_receipt_success, assert_xcmp_sent, log_all_roco_events,
        rococo, setup,
        slim::{Slender, Slim, SLENDER_PARA_ID, SLIM_PARA_ID},
        teleport_from_relay_to, transfer_to, ParaKind, RococoNet, ALICE,
    };
    use codec::Encode;
    use frame_support::{assert_ok, traits::Currency};
    use large::{Event, Origin, PolkadotXcm, System, XbiPortal};
    use pallet_xbi_portal::Message;
    use polkadot_primitives::v2::Id as ParaId;
    use sp_runtime::traits::{AccountIdConversion, Convert, UniqueSaturatedInto};
    use substrate_abi::{SubstrateAbiConverter, TryConvert};
    use xcm::{latest::prelude::*, VersionedMultiLocation, VersionedXcm};
    use xcm_emulator::TestExt;
    use xp_format::{Fees, Status, XbiFormat, XbiInstruction, XbiMetadata};

    fn log_all_events() {
        large::System::events()
            .iter()
            .for_each(|r| println!(">>> [Large] {:?}", r.event));
    }

    fn register_asset(id: u32, location: MultiLocation) {
        assert_ok!(large::AssetRegistry::register(
            large::Origin::root(),
            location.clone(),
            id
        ));
        assert_ok!(large::AssetRegistry::register_info(
            large::Origin::root(),
            pallet_asset_registry::AssetInfo::new(id, location.clone(), vec![]) // FIXME: add capabilities
        ));

        log_all_events();
        assert!(large::System::events().iter().any(|r| matches!(
            &r.event,
            large::Event::AssetRegistry(pallet_asset_registry::Event::Registered { asset_id, location: loc }) if asset_id == &id && &location == loc
        )));
        assert!(large::System::events().iter().any(|r| matches!(
            &r.event,
            large::Event::AssetRegistry(pallet_asset_registry::Event::Info { asset_id, location: loc }) if asset_id == &id && &location == loc
        )));
        System::reset_events();
    }

    fn create_asset(
        id: u32,
        name: &str,
        symbol: &str,
        decimals: u8,
        owner: Option<sp_runtime::AccountId32>,
        min_balance: u128,
    ) {
        assert_ok!(large::Assets::force_create(
            large::Origin::root(),
            id,
            owner.unwrap_or(ALICE),
            true,
            min_balance
        ));
        assert_ok!(large::Assets::set_metadata(
            large::Origin::signed(ALICE),
            id,
            name.encode(),
            symbol.encode(),
            decimals
        ));
        log_all_events();
        assert!(large::System::events().iter().any(|r| matches!(
            r.event,
            large::Event::Assets(pallet_assets::Event::ForceCreated { asset_id, .. }) if asset_id == id
        )));
        let n = name;
        let s = symbol;

        assert!(large::System::events().iter().any(|r| matches!(
            &r.event,
            large::Event::Assets(pallet_assets::Event::MetadataSet {
                asset_id,
                name,
                symbol,
                ..
            }) if asset_id == &id && name == &n.encode() && symbol == &s.encode()
        )));
        System::reset_events();
    }

    fn setup_default_assets() {
        let initial_balance = 100_000_000_000_000; // 100 ROC
        let large_sovereign: sp_runtime::AccountId32 =
            crate::para_id_to_account(ParaKind::Child(LARGE_PARA_ID));
        let slim_sovereign: sp_runtime::AccountId32 =
            crate::para_id_to_account(ParaKind::Child(SLIM_PARA_ID));

        transfer_to(large_sovereign, initial_balance);
        transfer_to(slim_sovereign, initial_balance);

        // create asset "xroc"
        Large::execute_with(|| {
            create_asset(1, "xRoc", "XROC", 12, None, 1);
            register_asset(1, MultiLocation::parent());
        });
        Slim::execute_with(|| {
            crate::slim::create_asset(1, "xRoc", "XROC", 12, None, 1, "Slim");
            crate::slim::register_asset(1, MultiLocation::parent(), "Slim");
        });
        Large::execute_with(|| {
            teleport_from_relay_to!(
                large,
                MultiLocation {
                    parents: 0,
                    interior: Junctions::X1(Junction::Parachain(LARGE_PARA_ID)),
                },
                MultiLocation {
                    parents: 1,
                    interior: Junctions::X1(Parachain(SLIM_PARA_ID)),
                },
                initial_balance / 10
            );
            System::reset_events();
        });
        println!(">>> [Rococo] proving execution success");
        RococoNet::execute_with(|| {
            log_all_roco_events();
            assert_relay_executed_upward!(Outcome::Complete(4_000_000_000));
            assert_deposit!(rococo, large::PolkadotXcm::check_account()); // Deposited to checking account on relay
            System::reset_events();
        });
    }

    #[test]
    fn ump() {
        setup();

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
        Large::execute_with(|| {
            assert_ok!(large::PolkadotXcm::send_xcm(
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
        setup();

        let remark = large::Call::System(frame_system::Call::<large::Runtime>::remark_with_event {
            remark: "Hello from Pumpkin!".as_bytes().to_vec(),
        });

        Large::execute_with(|| {
            assert_ok!(large::PolkadotXcm::send_xcm(
                Here,
                MultiLocation::new(1, X1(Parachain(2))),
                Xcm(vec![Transact {
                    origin_type: OriginKind::SovereignAccount,
                    require_weight_at_most: 10_000_000,
                    call: remark.encode().into(),
                }]),
            ));

            log_all_events();
        });

        Slender::execute_with(|| {
            use large::{Event, System};
            crate::slim::log_all_events("Slender");

            assert!(System::events().iter().any(|r| matches!(
                r.event,
                Event::System(frame_system::Event::Remarked { sender: _, hash: _ })
            )));
        });
    }

    #[test]
    fn xcmp_through_a_parachain() {
        setup();

        use large::{Call, PolkadotXcm, Runtime};

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
        Large::execute_with(|| {
            assert_ok!(PolkadotXcm::send_xcm(
                Here,
                MultiLocation::new(1, X1(Parachain(LARGE_PARA_ID))),
                Xcm(vec![Transact {
                    origin_type: OriginKind::SovereignAccount,
                    require_weight_at_most: 100_000_000,
                    call: send_xcm_to_t1rn.encode().into(),
                }]),
            ));
        });

        Large::execute_with(|| {
            use large::{Event, System};
            log_all_events();

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
            use large::{Event, System};
            // execution would fail, but good enough to check if the message is received
            crate::slim::log_all_events("Slender");

            assert!(System::events().iter().any(|r| matches!(
                r.event,
                Event::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail { .. })
            )));
        });
    }

    #[test]
    fn user_can_pay_for_transact_in_xroc() {
        setup();

        let large_initial_balance = 10_000_000_000_000; // 10 ROC
        let exec_fees_on_relay = 60_000_000_000; // 0.06 ROC
        let exec_fees_on_large = 1_000_000_000_000; // 1 XROC

        // create asset "xroc"
        Large::execute_with(|| {
            create_asset(1, "xRoc", "XROC", 12, None, 1);
            register_asset(1, MultiLocation::parent());
        });

        // transfer roc to sovereign for this chain
        transfer_to(
            crate::para_id_to_account(ParaKind::Child(LARGE_PARA_ID)),
            large_initial_balance,
        );

        println!(">>> [Large] instruct relay to teleport 10 roc to large");
        Large::execute_with(|| {
            assert_ok!(PolkadotXcm::send(
                large::Origin::root(), // Act on behalf of this parachain sovereign
                box MultiLocation::parent().versioned(),
                box VersionedXcm::V2(Xcm(vec![
                    Instruction::WithdrawAsset(MultiAssets::from(vec![MultiAsset {
                        id: AssetId::Concrete(MultiLocation::here()),
                        fun: Fungibility::Fungible(large_initial_balance),
                    }])),
                    BuyExecution {
                        fees: MultiAsset {
                            id: AssetId::Concrete(MultiLocation::here()),
                            fun: Fungibility::Fungible(exec_fees_on_relay),
                        },
                        weight_limit: Limited(5_000_000_000),
                    },
                    InitiateTeleport {
                        assets: MultiAssetFilter::Wild(All),
                        dest: MultiLocation {
                            parents: 0,
                            interior: Junctions::X1(Junction::Parachain(LARGE_PARA_ID)),
                        },
                        xcm: Xcm(vec![
                            BuyExecution {
                                fees: MultiAsset {
                                    id: AssetId::Concrete(MultiLocation::parent()),
                                    fun: Fungibility::Fungible(exec_fees_on_large),
                                },
                                weight_limit: Unlimited,
                            },
                            DepositAsset {
                                assets: Wild(All),
                                max_assets: 100000,
                                beneficiary: MultiLocation {
                                    parents: 0,
                                    interior: Junctions::X1(Junction::AccountId32 {
                                        network: NetworkId::Any,
                                        id: *ALICE.as_ref(),
                                    }),
                                },
                            },
                            RefundSurplus
                        ]),
                    },
                    RefundSurplus
                ])),
            ));
            log_all_events();
            assert_polkadot_sent!(large);
            System::reset_events();
        });

        println!(">>> [Rococo] proving execution success");
        RococoNet::execute_with(|| {
            log_all_roco_events();
            assert_relay_executed_upward!(Outcome::Complete(4_000_000_000));

            assert_withdrawal!(
                rococo,
                crate::para_id_to_account(ParaKind::Child(LARGE_PARA_ID)),
                large_initial_balance
            );
            assert_deposit!(rococo, large::PolkadotXcm::check_account()); // Deposited to checking account on relay

            System::reset_events();
        });

        println!(">>> [Large] proving alice on large has 10 XROC");
        Large::execute_with(|| {
            log_all_events();
            assert!(System::events().iter().any(|r| matches!(
				&r.event,
				Event::Assets(pallet_assets::Event::Issued {
					asset_id,
					owner,
					total_supply
				}) if asset_id == &1 && owner == &ALICE && total_supply >= &(large_initial_balance - exec_fees_on_relay)
			)));
            assert_relay_executed_downward!(large, Outcome::Complete(50));

            System::reset_events();
        });

        Slim::execute_with(|| {
            crate::slim::create_asset(1, "xRoc", "XROC", 12, None, 1, "Slim");
            crate::slim::register_asset(1, MultiLocation::parent(), "Slim");
        });

        println!(">>> [Large] Sending funds to slim to pay for xbi");
        Large::execute_with(|| {
            let funds_sent = 2_000_000_000_000;
            assert_ok!(PolkadotXcm::execute(
                Origin::signed(ALICE),
                box VersionedXcm::V2(Xcm(vec![
                    WithdrawAsset(MultiAssets::from(vec![MultiAsset {
                        id: AssetId::Concrete(MultiLocation::parent()),
                        fun: Fungibility::Fungible(funds_sent),
                    }])),
                    BuyExecution {
                        fees: MultiAsset {
                            id: AssetId::Concrete(MultiLocation::parent()),
                            fun: Fungibility::Fungible(exec_fees_on_relay),
                        },
                        weight_limit: Unlimited,
                    },
                    InitiateTeleport {
                        assets: MultiAssetFilter::Wild(All),
                        dest: MultiLocation {
                            parents: 1,
                            interior: Junctions::X1(Junction::Parachain(SLIM_PARA_ID)),
                        },
                        xcm: Xcm(vec![
                            BuyExecution {
                                fees: MultiAsset {
                                    id: AssetId::Concrete(MultiLocation::parent()),
                                    fun: Fungibility::Fungible(exec_fees_on_large),
                                },
                                weight_limit: Unlimited,
                            },
                            DepositAsset {
                                assets: Wild(All),
                                max_assets: funds_sent.ilog(12),
                                beneficiary: MultiLocation {
                                    parents: 1,
                                    interior: Junctions::X1(Parachain(LARGE_PARA_ID)),
                                },
                            },
                            RefundSurplus
                        ]),
                    },
                    RefundSurplus
                ])),
                funds_sent.unique_saturated_into(),
            ));
            log_all_events();

            assert_polkadot_attempted!(large);
            assert!(System::events().iter().any(|r| matches!(
                &r.event,
                Event::Assets(pallet_assets::Event::Burned { asset_id, owner, balance}) if asset_id == &1 && owner == &ALICE && balance == &funds_sent
            )));
            // We issue the asset to slim's checkin account for teleports
            assert!(System::events().iter().any(|r| matches!(
                &r.event,
                Event::Assets(pallet_assets::Event::Issued { asset_id, owner, total_supply}) if asset_id == &1 && owner == &slim::PolkadotXcm::check_account() && total_supply == &funds_sent
            )));
            System::reset_events();
        });

        println!(">>> [Slim] checking events for teleport");
        Slim::execute_with(|| {
            crate::slim::log_all_events("Slim");
            assert!(slim::System::events().iter().any(|r| matches!(
                &r.event,
                // slim::Event::Assets(pallet_assets::Event::Issued { asset_id, owner, total_supply}) if asset_id == &1 && owner == &crate::para_id_to_account(ParaKind::Child(LARGE_PARA_ID)) && total_supply == &1_000_000_000_000
                slim::Event::Assets(pallet_assets::Event::Issued { .. })
            )));
            assert_xcmp_receipt_success!(slim);
            slim::System::reset_events();
        });

        println!(">>> [Large] Sending xbi message to slim");
        Large::execute_with(|| {
            assert_ok!(XbiPortal::send(
                large::Origin::signed(ALICE),
                XbiFormat {
                    instr: XbiInstruction::CallEvm {
                        source: SubstrateAbiConverter::try_convert(ALICE).unwrap(),
                        target: substrate_abi::AccountId20::from_low_u64_be(1),
                        value: 0.into(),
                        input: b"hello world".to_vec(),
                        gas_limit: 5000000,
                        max_fee_per_gas: SubstrateAbiConverter::convert(0_u32),
                        max_priority_fee_per_gas: None,
                        nonce: None,
                        access_list: vec![]
                    },
                    metadata: XbiMetadata::new(
                        LARGE_PARA_ID,
                        SLIM_PARA_ID,
                        Default::default(),
                        Fees::new(Some(1), Some(90_000_000_000), Some(10_000_000_000)),
                        None,
                        Default::default(),
                        Default::default(),
                    ),
                }
            ));

            assert_xcmp_sent!(large);
            assert_xbi_sent!(large);

            log_all_events();
            System::reset_events();
        });

        println!(">>> [Slim] checking events for xbi message");
        Slim::execute_with(|| {
            crate::slim::log_all_events("Slim");
            assert_xbi_received!(slim);
            assert_xbi_instruction_handled!(slim);
            assert_xbi_request_handled!(slim);
            assert_xcmp_receipt_success!(slim);
            assert_xcmp_sent!(slim);
            assert_xbi_sent!(slim);
            assert_xcmp_receipt_success!(slim);
            slim::System::reset_events();
        });

        println!(">>> [Large] checking for xbi receipt");
        Large::execute_with(|| {
            log_all_events();
            assert_xbi_received!(large);
            assert_xcmp_receipt_success!(large);
            System::reset_events();
        });
    }

    #[test]
    fn slim_executes_an_evm_contract_on_large() {
        setup();
        setup_default_assets();

        println!(">>> [Slim] Sending xbi message to large");
        Slim::execute_with(|| {
            assert_ok!(slim::XbiPortal::send(
                slim::Origin::signed(ALICE),
                XbiFormat {
                    instr: XbiInstruction::CallEvm {
                        source: SubstrateAbiConverter::try_convert(ALICE).unwrap(),
                        target: substrate_abi::AccountId20::from_low_u64_be(1),
                        value: 0.into(),
                        input: b"hello world".to_vec(),
                        gas_limit: 5000000,
                        max_fee_per_gas: SubstrateAbiConverter::convert(0_u32),
                        max_priority_fee_per_gas: None,
                        nonce: None,
                        access_list: vec![]
                    },
                    metadata: XbiMetadata::new(
                        SLIM_PARA_ID,
                        LARGE_PARA_ID,
                        Default::default(),
                        Fees::new(Some(1), Some(90_000_000_000), Some(10_000_000_000)),
                        None,
                        Default::default(),
                        Default::default(),
                    ),
                }
            ));

            crate::slim::log_all_events("Slim");
            assert_xcmp_sent!(slim);
            assert_xbi_sent!(slim);
            slim::System::reset_events();
        });

        println!(">>> [Large] checking events for xbi message");
        Large::execute_with(|| {
            log_all_events();
            assert_xbi_received!(large);
            assert_xbi_request_handled!(large);
            assert_xbi_instruction_handled!(large);
            assert_xcmp_receipt_success!(large);
            assert_xcmp_sent!(large);
            assert_xbi_sent!(large, Status::Success);
            assert_xcmp_receipt_success!(large);
            System::reset_events();
        });

        Slim::execute_with(|| {
            crate::slim::log_all_events("Slim");
            slim::System::reset_events();
        });
    }

    #[test]
    fn slim_executes_a_wasm_contract_on_large() {
        setup();
        setup_default_assets();

        Large::execute_with(|| {
            let contract_path = "fixtures/transfer_return_code.wat";
            let wasm = wat::parse_file(contract_path).expect("Failed to parse file");
            assert_ok!(large::Contracts::instantiate_with_code(
                Origin::signed(ALICE),
                0,
                100_000_000_000,
                None,
                wasm,
                vec![],
                vec![],
            ));
            log_all_events();
            System::reset_events();
        });

        println!(">>> [Slim] Sending xbi message to large");
        Slim::execute_with(|| {
            assert_ok!(slim::XbiPortal::send(
                slim::Origin::signed(ALICE),
                XbiFormat {
                    instr: XbiInstruction::CallWasm {
                        dest: hex_literal::hex!(
                            "18a84a38cff91f3345a66802803f8959d11d4d2315a082bfeb2a49ce72b2577f"
                        )
                        .into(),
                        value: 0,
                        gas_limit: 500_000_000_000,
                        storage_deposit_limit: None,
                        data: b"".to_vec()
                    },
                    metadata: XbiMetadata::new(
                        SLIM_PARA_ID,
                        LARGE_PARA_ID,
                        Default::default(),
                        Fees::new(Some(1), Some(90_000_000_000), Some(10_000_000_000)),
                        None,
                        Default::default(),
                        Default::default(),
                    ),
                }
            ));
            crate::slim::log_all_events("Slim");
            assert_xcmp_sent!(slim);
            assert_xbi_sent!(slim);
            slim::System::reset_events();
        });

        println!(">>> [Large] checking events for xbi message");
        Large::execute_with(|| {
            log_all_events();
            assert_xbi_received!(large);
            assert_xbi_request_handled!(large);
            assert_xbi_instruction_handled!(large);
            assert_xcmp_receipt_success!(large);
            assert_xcmp_sent!(large);
            assert_xbi_sent!(large, Status::Success);
            assert_xcmp_receipt_success!(large);
            System::reset_events();
        });

        println!(">>> [Slim] Checking slim events");
        Slim::execute_with(|| {
            crate::slim::log_all_events("Slim");
            slim::System::reset_events();
        });
    }

    #[test]
    fn user_cannot_exhaust_more_than_provided_gas() {
        setup();
        setup_default_assets();

        Large::execute_with(|| {
            let contract_path = "fixtures/transfer_return_code.wat";
            let wasm = wat::parse_file(contract_path).expect("Failed to parse file");
            assert_ok!(large::Contracts::instantiate_with_code(
                Origin::signed(ALICE),
                0,
                100_000_000_000,
                None,
                wasm,
                vec![],
                vec![],
            ));
            log_all_events();
            System::reset_events();
        });

        println!(">>> [Slim] Sending xbi message to large");
        Slim::execute_with(|| {
            assert_ok!(slim::XbiPortal::send(
                slim::Origin::signed(ALICE),
                XbiFormat {
                    instr: XbiInstruction::CallWasm {
                        dest: hex_literal::hex!(
                            "18a84a38cff91f3345a66802803f8959d11d4d2315a082bfeb2a49ce72b2577f"
                        )
                        .into(),
                        value: 0,
                        gas_limit: 500_000_000_000, // TODO: decide how we pass this through, really it should come from XBIMetadata
                        storage_deposit_limit: None,
                        data: b"".to_vec()
                    },
                    metadata: XbiMetadata::new(
                        SLIM_PARA_ID,
                        LARGE_PARA_ID,
                        Default::default(),
                        Fees::new(Some(1), Some(100_000), Some(10_000_000_000)),
                        None,
                        Default::default(),
                        Default::default(),
                    ),
                }
            ));

            crate::slim::log_all_events("Slim");
            assert_xcmp_sent!(slim);
            assert_xbi_sent!(slim);
            slim::System::reset_events();
        });

        println!(">>> [Large] checking events for xbi message");
        Large::execute_with(|| {
            log_all_events();
            assert_xbi_received!(large);
            assert_xbi_request_handled!(large);
            assert_xbi_instruction_handled!(large);
            assert_xcmp_receipt_success!(large);
            assert_xcmp_sent!(large);
            assert_xbi_sent!(large, Status::ExecutionLimitExceeded);
            assert_xcmp_receipt_success!(large);
            System::reset_events();
        });

        Slim::execute_with(|| {
            crate::slim::log_all_events("Slim");
            slim::System::reset_events();
        });
    }

    // TODO:
    #[test]
    fn user_is_refunded_most_of_fees_on_target_fail() {}

    // TODO:
    #[test]
    fn xbi_call_can_timeout() {
        // most of the call is refunded minus transport fees
    }

    // TODO:
    #[test]
    fn xbi_call_can_short_circuit_on_costs_overflow() {
        // most of the call is refunded minus transport fees
    }
}
