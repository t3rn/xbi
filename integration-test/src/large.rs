use crate::{ALICE, INITIAL_BALANCE};
use frame_support::traits::GenesisBuild;

pub const LARGE_PARA_ID: u32 = 3;

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
        balances: vec![(ALICE, INITIAL_BALANCE)],
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
    use frame_support::weights::Weight;

    use crate::{
        assert_asset_burned, assert_asset_issued, assert_deposit, assert_polkadot_attempted,
        assert_polkadot_sent, assert_relay_executed_downward, assert_relay_executed_upward,
        assert_response_stored, assert_withdrawal, assert_xbi_instruction_handled,
        assert_xbi_received, assert_xbi_request_handled, assert_xbi_sent,
        assert_xcmp_receipt_success, assert_xcmp_sent, log_all_roco_events, para_id_to_account,
        rococo, setup,
        slim::{SLENDER_PARA_ID, SLIM_PARA_ID},
        teleport_from_relay_to, transfer_to, Large, ParaKind, RococoNet, Slender, Slim, ALICE,
        CONTRACT_CALLER,
    };
    use codec::Encode;
    use frame_support::{assert_ok, traits::Currency};
    use large::{PolkadotXcm, RuntimeEvent, RuntimeOrigin, System, XbiPortal};
    use pallet_xbi_portal::Message;
    use polkadot_primitives::v2::Id as ParaId;
    use sp_runtime::traits::{AccountIdConversion, Convert, UniqueSaturatedInto};
    use substrate_abi::{SubstrateAbiConverter, TryConvert};
    use xcm::{latest::prelude::*, VersionedMultiLocation, VersionedXcm};
    use xcm_emulator::TestExt;
    use xp_format::{Fees, Status, Timeouts, XbiFormat, XbiInstruction, XbiMetadata};

    const ASSET_ID: u32 = 1;
    const EXEC_COST: u128 = 90_000_000_000_000_000_000_000;
    const GAS_LIMIT: u64 = 90_000_000_000_000_000;
    const NOTIFICATION_COST: u128 = 10_000_000_000_000_000_000_000;
    const NOTIFICATION_WEIGHT: u128 = 100_000_000_000_000_000_000;
    // Weight for the contract call as identity fee
    const WASM_EXECUTION_FEE: u128 = 32063234_000_000_000_000;
    const BALANCE_CUSTODIAN: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([64u8; 32]);

    fn log_all_events() {
        large::System::events()
            .iter()
            .for_each(|r| println!(">>> [Large] {:?}", r.event));
    }

    fn init_wasm_fixture() {
        Large::execute_with(|| {
            let contract_path = "fixtures/transfer_return_code.wat";
            let wasm = wat::parse_file(contract_path).expect("Failed to parse file");
            assert_ok!(large::Contracts::instantiate_with_code(
                RuntimeOrigin::signed(ALICE),
                0,
                100_000_000_000_000.into(),
                None,
                wasm,
                vec![],
                vec![],
            ));
            log_all_events();
            System::reset_events();
        });
    }

    fn register_asset(id: u32, location: MultiLocation) {
        assert_ok!(large::AssetRegistry::register(
            large::RuntimeOrigin::root(),
            location.clone(),
            id
        ));
        assert_ok!(large::AssetRegistry::register_info(
            large::RuntimeOrigin::root(),
            pallet_asset_registry::AssetInfo::new(id, location.clone(), vec![]) // FIXME: add capabilities
        ));

        log_all_events();
        assert!(large::System::events().iter().any(|r| matches!(
            &r.event,
            large::RuntimeEvent::AssetRegistry(pallet_asset_registry::Event::Registered { asset_id, location: loc }) if asset_id == &id && &location == loc
        )));
        assert!(large::System::events().iter().any(|r| matches!(
            &r.event,
            large::RuntimeEvent::AssetRegistry(pallet_asset_registry::Event::Info { asset_id, location: loc }) if asset_id == &id && &location == loc
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
            large::RuntimeOrigin::root(),
            id,
            owner.unwrap_or(ALICE),
            true,
            min_balance
        ));
        assert_ok!(large::Assets::set_metadata(
            large::RuntimeOrigin::signed(ALICE),
            id,
            name.encode(),
            symbol.encode(),
            decimals
        ));
        log_all_events();
        assert!(large::System::events().iter().any(|r| matches!(
            r.event,
            large::RuntimeEvent::Assets(pallet_assets::Event::ForceCreated { asset_id, .. }) if asset_id == id
        )));
        let n = name;
        let s = symbol;

        assert!(large::System::events().iter().any(|r| matches!(
            &r.event,
            large::RuntimeEvent::Assets(pallet_assets::Event::MetadataSet {
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
            create_asset(ASSET_ID, "xRoc", "XROC", 12, None, 1);
            register_asset(ASSET_ID, MultiLocation::parent());
        });
        Slim::execute_with(|| {
            crate::slim::create_asset(ASSET_ID, "xRoc", "XROC", 12, None, 1, "Slim");
            crate::slim::register_asset(ASSET_ID, MultiLocation::parent(), "Slim");
            crate::slim::mint_asset(ASSET_ID, ALICE, initial_balance);
            crate::slim::mint_asset(ASSET_ID, CONTRACT_CALLER, initial_balance * 100);
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
            assert_relay_executed_upward!(Outcome::Complete(Weight::from_ref_time(767529000)));
            assert_deposit!(rococo, large::PolkadotXcm::check_account()); // Deposited to checking account on relay
            rococo::System::reset_events();
        });

        Large::execute_with(|| {
            log_all_events();
            large::System::reset_events();
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

        let remark = rococo_runtime::RuntimeCall::System(frame_system::Call::<
            rococo_runtime::Runtime,
        >::remark_with_event {
            remark: "Hello from Pumpkin!".as_bytes().to_vec(),
        });
        Large::execute_with(|| {
            assert_ok!(large::PolkadotXcm::send_xcm(
                Here,
                Parent,
                Xcm(vec![Transact {
                    origin_kind: OriginKind::SovereignAccount,
                    require_weight_at_most: (INITIAL_BALANCE as u64).into(),
                    call: remark.encode().into(),
                }]),
            ));
        });

        RococoNet::execute_with(|| {
            use rococo_runtime::{RuntimeEvent, System};

            System::events()
                .iter()
                .for_each(|r| println!(">>> [RelayChain] {:?}", r.event));

            assert!(System::events().iter().any(|r| matches!(
                r.event,
                RuntimeEvent::System(frame_system::Event::NewAccount { account: _ })
            )));
        });
    }

    #[test]
    fn xcmp() {
        setup();

        let remark =
            large::RuntimeCall::System(frame_system::Call::<large::Runtime>::remark_with_event {
                remark: "Hello from Pumpkin!".as_bytes().to_vec(),
            });

        Large::execute_with(|| {
            assert_ok!(large::PolkadotXcm::send_xcm(
                Here,
                MultiLocation::new(1, X1(Parachain(2))),
                Xcm(vec![Transact {
                    origin_kind: OriginKind::SovereignAccount,
                    require_weight_at_most: 10_000_000.into(),
                    call: remark.encode().into(),
                }]),
            ));

            log_all_events();
        });

        Slender::execute_with(|| {
            use large::{RuntimeEvent, System};
            crate::slim::log_all_events("Slender");

            assert!(System::events().iter().any(|r| matches!(
                r.event,
                RuntimeEvent::System(frame_system::Event::Remarked { sender: _, hash: _ })
            )));
        });
    }

    #[test]
    fn xcmp_through_a_parachain() {
        setup();

        use large::{PolkadotXcm, Runtime, RuntimeCall};

        // The message goes through: Pumpkin --> Mushroom --> Octopus
        let remark = RuntimeCall::System(frame_system::Call::<Runtime>::remark_with_event {
            remark: "Hello from Pumpkin!".as_bytes().to_vec(),
        });

        let send_xcm_to_t1rn = RuntimeCall::PolkadotXcm(pallet_xcm::Call::<Runtime>::send {
            dest: Box::new(VersionedMultiLocation::V3(MultiLocation::new(
                1,
                X1(Parachain(SLENDER_PARA_ID)),
            ))),
            message: Box::new(VersionedXcm::V3(Xcm(vec![Transact {
                origin_kind: OriginKind::SovereignAccount,
                require_weight_at_most: 10_000_000.into(),
                call: remark.encode().into(),
            }]))),
        });
        Large::execute_with(|| {
            assert_ok!(PolkadotXcm::send_xcm(
                Here,
                MultiLocation::new(1, X1(Parachain(LARGE_PARA_ID))),
                Xcm(vec![Transact {
                    origin_kind: OriginKind::SovereignAccount,
                    require_weight_at_most: 100_000_000_000.into(),
                    call: send_xcm_to_t1rn.encode().into(),
                }]),
            ));
        });

        Large::execute_with(|| {
            use large::{RuntimeEvent, System};
            log_all_events();

            assert!(System::events().iter().any(|r| matches!(
                r.event,
                RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. })
            )));
            assert!(System::events().iter().any(|r| matches!(
                r.event,
                RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent(_, _, _))
            )));
        });

        Slender::execute_with(|| {
            use large::{RuntimeEvent, System};
            // execution would fail, but good enough to check if the message is received
            crate::slim::log_all_events("Slender");

            assert!(System::events().iter().any(|r| matches!(
                r.event,
                RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail { .. })
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
                large::RuntimeOrigin::root(), // Act on behalf of this parachain sovereign
                box MultiLocation::parent().into_versioned(),
                box VersionedXcm::V3(Xcm(vec![
                    Instruction::WithdrawAsset(MultiAssets::from(vec![MultiAsset {
                        id: AssetId::Concrete(MultiLocation::here()),
                        fun: Fungibility::Fungible(large_initial_balance),
                    }])),
                    BuyExecution {
                        fees: MultiAsset {
                            id: AssetId::Concrete(MultiLocation::here()),
                            fun: Fungibility::Fungible(exec_fees_on_relay),
                        },
                        weight_limit: Limited(5_000_000_000.into()),
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
                                beneficiary: MultiLocation {
                                    parents: 0,
                                    interior: Junctions::X1(Junction::AccountId32 {
                                        network: None,
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
            assert_relay_executed_upward!(Outcome::Complete(Weight::from_ref_time(767529000)));

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
				RuntimeEvent::Assets(pallet_assets::Event::Issued {
					asset_id,
					owner,
					amount
				}) if asset_id == &ASSET_ID && owner == &ALICE && amount >= &(large_initial_balance - exec_fees_on_relay)
			)));
            assert_relay_executed_downward!(large, Outcome::Complete(Weight::from_ref_time(50)));

            System::reset_events();
        });

        Slim::execute_with(|| {
            crate::slim::create_asset(ASSET_ID, "xRoc", "XROC", 12, None, 1, "Slim");
            crate::slim::register_asset(ASSET_ID, MultiLocation::parent(), "Slim");
        });

        println!(">>> [Large] Sending funds to slim to pay for xbi");
        Large::execute_with(|| {
            let funds_sent = 2_000_000_000_000;
            assert_ok!(PolkadotXcm::execute(
                RuntimeOrigin::signed(ALICE),
                box VersionedXcm::V3(Xcm(vec![
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
                Weight::from_ref_time(funds_sent.unique_saturated_into()),
            ));
            log_all_events();

            assert_polkadot_attempted!(large);
            assert!(System::events().iter().any(|r| matches!(
                &r.event,
                RuntimeEvent::Assets(pallet_assets::Event::Burned { asset_id, owner, balance}) if asset_id == &ASSET_ID && owner == &ALICE && balance == &funds_sent
            )));
            // We issue the asset to slim's checkin account for teleports
            assert!(System::events().iter().any(|r| matches!(
                &r.event,
                RuntimeEvent::Assets(pallet_assets::Event::Issued { asset_id, owner, amount}) if asset_id == &ASSET_ID && owner == &slim::PolkadotXcm::check_account() && amount == &funds_sent
            )));
            System::reset_events();
        });

        println!(">>> [Slim] checking events for teleport");
        Slim::execute_with(|| {
            crate::slim::log_all_events("Slim");
            assert!(slim::System::events().iter().any(|r| matches!(
                &r.event,
                slim::RuntimeEvent::Assets(pallet_assets::Event::Issued { .. })
            )));
            assert_xcmp_receipt_success!(slim);
            slim::System::reset_events();
        });

        println!(">>> [Large] Sending xbi message to slim");
        Large::execute_with(|| {
            assert_ok!(XbiPortal::send(
                large::RuntimeOrigin::signed(ALICE),
                xp_channel::ExecutionType::Sync,
                XbiFormat {
                    instr: XbiInstruction::CallWasm {
                        dest: hex_literal::hex!(
                            "4e519d0d228bc7f0cedcfc3e1707696c97d9430645ab7cb1b2aece11ce7fe2e0"
                        )
                        .into(),
                        value: 0,
                        gas_limit: 1_000_000,
                        storage_deposit_limit: None,
                        data: b"".to_vec()
                    },
                    metadata: XbiMetadata::new(
                        LARGE_PARA_ID,
                        SLIM_PARA_ID,
                        Default::default(),
                        Fees::new(Some(ASSET_ID), Some(EXEC_COST), Some(NOTIFICATION_COST)),
                        None,
                        Default::default(),
                        Default::default(),
                    ),
                }
            ));

            log_all_events();

            assert_xcmp_sent!(large);
            assert_xbi_sent!(large);
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
            assert_response_stored!(large, Status::Success);
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
                slim::RuntimeOrigin::signed(ALICE),
                xp_channel::ExecutionType::Sync,
                XbiFormat {
                    instr: XbiInstruction::CallEvm {
                        target: substrate_abi::AccountId20::from_low_u64_be(1),
                        value: 0.into(),
                        input: b"hello world".to_vec(),
                        gas_limit: GAS_LIMIT,
                        max_fee_per_gas: SubstrateAbiConverter::convert(0_u32),
                        max_priority_fee_per_gas: None,
                        nonce: None,
                        access_list: vec![]
                    },
                    metadata: XbiMetadata::new(
                        SLIM_PARA_ID,
                        LARGE_PARA_ID,
                        Default::default(),
                        Fees::new(Some(ASSET_ID), Some(EXEC_COST), Some(NOTIFICATION_COST)),
                        None,
                        Default::default(),
                        Default::default(),
                    ),
                }
            ));

            crate::slim::log_all_events("Slim");
            assert_xcmp_sent!(slim);
            // Assert owner paid for the execution fees
            assert_asset_burned!(slim, ASSET_ID, ALICE, EXEC_COST + NOTIFICATION_COST);
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

        println!(">>> [Slim] checking events for resulting costs");
        Slim::execute_with(|| {
            crate::slim::log_all_events("Slim");
            assert_response_stored!(slim, Status::Success);
            slim::System::reset_events();
        });
    }

    #[test]
    fn slim_executes_a_wasm_contract_on_large() {
        setup();
        setup_default_assets();
        init_wasm_fixture();

        println!(">>> [Slim] Sending xbi message to large");
        Slim::execute_with(|| {
            assert_ok!(slim::XbiPortal::send(
                slim::RuntimeOrigin::signed(ALICE),
                xp_channel::ExecutionType::Sync,
                XbiFormat {
                    instr: XbiInstruction::CallWasm {
                        dest: hex_literal::hex!(
                            "4e519d0d228bc7f0cedcfc3e1707696c97d9430645ab7cb1b2aece11ce7fe2e0"
                        )
                        .into(),
                        value: 0,
                        gas_limit: 1_000_000,
                        storage_deposit_limit: None,
                        data: b"".to_vec()
                    },
                    metadata: XbiMetadata::new(
                        SLIM_PARA_ID,
                        LARGE_PARA_ID,
                        Default::default(),
                        Fees::new(Some(ASSET_ID), Some(EXEC_COST), Some(NOTIFICATION_COST)),
                        None,
                        Default::default(),
                        Default::default(),
                    ),
                }
            ));
            crate::slim::log_all_events("Slim");
            assert_xcmp_sent!(slim);
            // Assert owner paid for the execution fees
            assert_asset_burned!(slim, ASSET_ID, ALICE, EXEC_COST + NOTIFICATION_COST);
            println!(
                ">>> [Slim] checking events for xbi message checkin account {:#?}",
                slim::PolkadotXcm::check_account()
            );
            // Assert that balance custodian received the transfer
            assert_asset_issued!(
                slim,
                ASSET_ID,
                BALANCE_CUSTODIAN,
                EXEC_COST + NOTIFICATION_COST
            );
            assert_xbi_sent!(slim);
            slim::System::reset_events();
        });

        println!(">>> [Large] checking events for xbi message");
        Large::execute_with(|| {
            log_all_events();
            // Assert we burned the max fees on behalf of SLIM
            assert_asset_burned!(
                large,
                ASSET_ID,
                para_id_to_account(ParaKind::Sibling(SLIM_PARA_ID)),
                EXEC_COST + NOTIFICATION_COST
            );
            assert_xbi_received!(large);
            assert_xbi_request_handled!(large);
            assert_xcmp_sent!(large);
            assert_xbi_instruction_handled!(large);
            assert_xcmp_receipt_success!(large);
            assert_xbi_sent!(large, Status::Success);
            assert_xcmp_receipt_success!(large);
            System::reset_events();
        });

        println!(">>> [Slim] Checking slim events");
        Slim::execute_with(|| {
            crate::slim::log_all_events("Slim");
            assert_response_stored!(slim, Status::Success);
            // Assert that alice was returned her fees - execution fees
            assert_asset_issued!(
                slim,
                ASSET_ID,
                ALICE,
                (EXEC_COST + NOTIFICATION_COST) - WASM_EXECUTION_FEE - NOTIFICATION_WEIGHT
            );
            slim::System::reset_events();
        });
    }

    #[test]
    fn user_cannot_exhaust_more_than_provided_gas() {
        setup();
        setup_default_assets();
        init_wasm_fixture();

        println!(">>> [Slim] Sending xbi message to large");
        Slim::execute_with(|| {
            assert_ok!(slim::XbiPortal::send(
                slim::RuntimeOrigin::signed(ALICE),
                xp_channel::ExecutionType::Sync,
                XbiFormat {
                    instr: XbiInstruction::CallWasm {
                        dest: hex_literal::hex!(
                            "4e519d0d228bc7f0cedcfc3e1707696c97d9430645ab7cb1b2aece11ce7fe2e0"
                        )
                        .into(),
                        value: 0,
                        gas_limit: 1_000_000, // TODO: decide how we pass this through, really it should come from XBIMetadata
                        storage_deposit_limit: None,
                        data: b"".to_vec()
                    },
                    metadata: XbiMetadata::new(
                        SLIM_PARA_ID,
                        LARGE_PARA_ID,
                        Default::default(),
                        Fees::new(Some(1), Some(NOTIFICATION_COST), Some(EXEC_COST)),
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
            assert_xcmp_sent!(large);
            assert_xbi_received!(large);
            assert_xbi_request_handled!(large);
            assert_xbi_instruction_handled!(large);
            assert_xcmp_receipt_success!(large);
            assert_xbi_sent!(large, Status::ExecutionLimitExceeded);
            assert_xcmp_receipt_success!(large);
            System::reset_events();
        });

        Slim::execute_with(|| {
            crate::slim::log_all_events("Slim");
            assert_response_stored!(slim, Status::ExecutionLimitExceeded);
            slim::System::reset_events();
        });
    }

    // FIXME: this test never completes with 2023-06-15T10:52:42.452Z ERROR [xbi] Failed to send xcm request: Transport("NoChannel") #[test]
    fn slim_executes_an_evm_contract_on_large_async() {
        setup();
        setup_default_assets();

        println!(">>> [Slim] Queueing xbi message");
        Slim::execute_with(|| {
            assert_ok!(slim::XbiPortal::send(
                slim::RuntimeOrigin::signed(ALICE),
                xp_channel::ExecutionType::Async,
                XbiFormat {
                    instr: XbiInstruction::CallEvm {
                        target: substrate_abi::AccountId20::from_low_u64_be(1),
                        value: 0.into(),
                        input: b"hello world".to_vec(),
                        gas_limit: GAS_LIMIT,
                        max_fee_per_gas: SubstrateAbiConverter::convert(0_u32),
                        max_priority_fee_per_gas: None,
                        nonce: None,
                        access_list: vec![]
                    },
                    metadata: XbiMetadata::new(
                        SLIM_PARA_ID,
                        LARGE_PARA_ID,
                        Default::default(),
                        Fees::new(Some(1), Some(90_000_000_000), Some(EXEC_COST)),
                        None,
                        Default::default(),
                        Default::default(),
                    ),
                }
            ));

            crate::slim::log_all_events("Slim");
            slim::System::reset_events();
        });

        println!(">>> [Slim] Processing queue");
        Slim::execute_with(|| {
            assert_ok!(slim::XbiPortal::process_queue(slim::RuntimeOrigin::root()));
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
            assert_response_stored!(slim, Status::Success);
            slim::System::reset_events();
        });
    }

    #[test]
    fn slim_executes_a_wasm_contract_on_large_async() {
        setup();
        setup_default_assets();
        init_wasm_fixture();

        println!(">>> [Slim] Queueing xbi message");
        Slim::execute_with(|| {
            assert_ok!(slim::XbiPortal::send(
                slim::RuntimeOrigin::signed(ALICE),
                xp_channel::ExecutionType::Async,
                XbiFormat {
                    instr: XbiInstruction::CallWasm {
                        dest: hex_literal::hex!(
                            "4e519d0d228bc7f0cedcfc3e1707696c97d9430645ab7cb1b2aece11ce7fe2e0"
                        )
                        .into(),
                        value: 0,
                        gas_limit: GAS_LIMIT,
                        storage_deposit_limit: None,
                        data: b"".to_vec()
                    },
                    metadata: XbiMetadata::new(
                        SLIM_PARA_ID,
                        LARGE_PARA_ID,
                        Default::default(),
                        Fees::new(Some(1), Some(EXEC_COST), Some(NOTIFICATION_COST)),
                        None,
                        Default::default(),
                        Default::default(),
                    ),
                }
            ));
            crate::slim::log_all_events("Slim");
            // Assert owner paid for the execution fees

            assert_asset_burned!(slim, ASSET_ID, ALICE, 100000000000);
            // Assert that checkin account claimed the withdrawal
            assert_asset_issued!(slim, ASSET_ID, BALANCE_CUSTODIAN, 100000000000);
            slim::System::reset_events();
        });

        println!(">>> [Slim] Processing queue");
        Slim::execute_with(|| {
            assert_ok!(slim::XbiPortal::process_queue(slim::RuntimeOrigin::root()));
            crate::slim::log_all_events("Slim");
            assert_xcmp_sent!(slim);
            assert_xbi_sent!(slim);
            slim::System::reset_events();
        });

        println!(">>> [Large] checking events for xbi message");
        Large::execute_with(|| {
            log_all_events();
            assert_xcmp_sent!(large);
            assert_xbi_received!(large);
            assert_xbi_request_handled!(large);
            // assert_xbi_instruction_handled!(large);
            assert_xcmp_receipt_success!(large);
            // assert_xbi_sent!(large, Status::Success);
            assert_xcmp_receipt_success!(large);
            System::reset_events();
        });

        println!(">>> [Slim] Checking slim events");
        Slim::execute_with(|| {
            crate::slim::log_all_events("Slim");
            assert_response_stored!(slim, Status::Success);
            slim::System::reset_events();
        });
    }

    #[test]
    fn user_cannot_exhaust_more_than_provided_gas_async() {
        setup();
        setup_default_assets();
        init_wasm_fixture();

        println!(">>> [Slim] Queueing xbi message");
        Slim::execute_with(|| {
            assert_ok!(slim::XbiPortal::send(
                slim::RuntimeOrigin::signed(ALICE),
                xp_channel::ExecutionType::Async,
                XbiFormat {
                    instr: XbiInstruction::CallWasm {
                        dest: hex_literal::hex!(
                            "4e519d0d228bc7f0cedcfc3e1707696c97d9430645ab7cb1b2aece11ce7fe2e0"
                        )
                        .into(),
                        value: 0,
                        gas_limit: 1_000_000, // TODO: decide how we pass this through, really it should come from XBIMetadata
                        storage_deposit_limit: None,
                        data: b"".to_vec()
                    },
                    metadata: XbiMetadata::new(
                        SLIM_PARA_ID,
                        LARGE_PARA_ID,
                        Default::default(),
                        Fees::new(Some(ASSET_ID), Some(NOTIFICATION_COST), Some(EXEC_COST)),
                        None,
                        Default::default(),
                        Default::default(),
                    ),
                }
            ));

            crate::slim::log_all_events("Slim");
            slim::System::reset_events();
        });

        println!(">>> [Slim] Processing queue");
        Slim::execute_with(|| {
            assert_ok!(slim::XbiPortal::process_queue(slim::RuntimeOrigin::root()));
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
            assert_response_stored!(slim, Status::ExecutionLimitExceeded);
            slim::System::reset_events();
        });
    }

    #[test]
    fn user_is_refunded_most_of_fees_on_target_fail() {
        setup();
        setup_default_assets();
        init_wasm_fixture();

        let made_up_dest = sp_runtime::AccountId32::new([99u8; 32]);

        println!(">>> [Slim] Queueing xbi message");
        Slim::execute_with(|| {
            assert_ok!(XbiPortal::send(
                RuntimeOrigin::signed(ALICE),
                xp_channel::ExecutionType::Async,
                XbiFormat {
                    instr: XbiInstruction::CallWasm {
                        dest: made_up_dest,
                        value: 5,
                        gas_limit: 1_000_000,
                        storage_deposit_limit: None,
                        data: b"".to_vec()
                    },
                    metadata: XbiMetadata::new(
                        SLIM_PARA_ID,
                        LARGE_PARA_ID,
                        Timeouts {
                            sent: 10.into(),
                            delivered: 20.into(),
                            executed: 30.into(),
                            ..Default::default()
                        },
                        Fees::new(Some(ASSET_ID), Some(EXEC_COST), Some(NOTIFICATION_COST)),
                        Default::default(),
                        0,
                        Default::default(),
                    ),
                }
            ));

            assert_ok!(slim::XbiPortal::process_queue(slim::RuntimeOrigin::root()));

            crate::slim::log_all_events("Slim");
            // Assert owner paid for the execution fees
            assert_asset_burned!(slim, ASSET_ID, ALICE, EXEC_COST + NOTIFICATION_COST);
            // Balance custodian received
            assert_asset_issued!(
                slim,
                ASSET_ID,
                BALANCE_CUSTODIAN,
                EXEC_COST + NOTIFICATION_COST
            );
            slim::System::reset_events();
        });

        println!(">>> [Large] checking large for message fees application");
        Large::execute_with(|| {
            log_all_events();
            // Assert owner paid for the execution fees
            assert_asset_burned!(
                large,
                ASSET_ID,
                para_id_to_account(ParaKind::Sibling(SLIM_PARA_ID)),
                EXEC_COST + NOTIFICATION_COST
            );
            System::reset_events();
        });

        println!(">>> [Slim] Checking result");
        Slim::execute_with(|| {
            crate::slim::log_all_events("Slim");
            assert_asset_burned!(
                slim,
                ASSET_ID,
                BALANCE_CUSTODIAN,
                EXEC_COST + NOTIFICATION_COST - NOTIFICATION_WEIGHT
            );
            // TODO: figure out why BuyExecution is just dumped
            assert_asset_issued!(
                slim,
                ASSET_ID,
                ALICE,
                EXEC_COST + NOTIFICATION_COST - NOTIFICATION_WEIGHT
            );
            slim::System::reset_events();
        });
    }

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
