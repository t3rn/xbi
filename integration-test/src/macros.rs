#[macro_export]
macro_rules! assert_polkadot_sent {
    ($runtime:ident) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent(_, _, _))
        )));
    };
}

#[macro_export]
macro_rules! assert_polkadot_attempted {
    ($runtime:ident) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Attempted(Outcome::Complete(_)))
        )));
    };
    ($runtime:ident, $outcome:expr) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Attempted($outcome))
        )));
    };
}

#[macro_export]
macro_rules! assert_xcmp_sent {
    ($runtime:ident) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::XcmpQueue(
                cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. }
            )
        )));
    };
}

#[macro_export]
macro_rules! assert_xcmp_receipt_success {
    ($runtime:ident) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Success { .. })
        )));
    };
}

#[macro_export]
macro_rules! assert_xbi_sent {
    ($runtime:ident) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
                    &r.event,
                    $runtime::RuntimeEvent::XbiPortal(pallet_xbi_portal::Event::XbiMessageSent { .. })
                )));
    };
    ($runtime:ident, $status:expr) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
                    &r.event,
                    $runtime::RuntimeEvent::XbiPortal(pallet_xbi_portal::Event::XbiMessageSent {
                            msg: Message::Response(xp_format::XbiResult { status, .. }, _)
                        }) if status == &$status
                )));
    };
}

#[macro_export]
macro_rules! assert_response_stored {
    ($runtime:ident) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
                    &r.event,
                    $runtime::RuntimeEvent::XbiPortal(pallet_xbi_portal::Event::ResponseStored { .. })
                )));
    };
    ($runtime:ident, $status:expr) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
                    &r.event,
                    $runtime::RuntimeEvent::XbiPortal(pallet_xbi_portal::Event::ResponseStored {
                        result: xp_format::XbiResult { status, ..},
                        ..
                    }) if status == &$status
                )));
    };
}

#[macro_export]
macro_rules! assert_xbi_received {
    ($runtime:ident) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::XbiPortal(pallet_xbi_portal::Event::XbiMessageReceived { .. })
        )));
    };
}
#[macro_export]
macro_rules! assert_xbi_instruction_handled {
    ($runtime:ident) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::XbiPortal(
                pallet_xbi_portal::Event::XbiInstructionHandled { .. }
            )
        )));
    };
}
#[macro_export]
macro_rules! assert_xbi_request_handled {
    ($runtime:ident) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::XbiPortal(pallet_xbi_portal::Event::XbiRequestHandled { .. })
        )));
    };
}

#[macro_export]
macro_rules! assert_relay_executed_upward {
    ($outcome:expr) => {
        assert!(rococo::System::events().iter().any(|r| matches!(
            &r.event,
            rococo::RuntimeEvent::Ump(polkadot_runtime_parachains::ump::Event::ExecutedUpward(
                _,
                outcome,
            )) if outcome == &$outcome // weight
        )));
    };
}
#[macro_export]
macro_rules! assert_relay_executed_downward {
    ($runtime:ident, $outcome:expr) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::DmpQueue($runtime::cumulus_pallet_dmp_queue::Event::ExecutedDownward {
                outcome,
                ..
            }) if outcome == &$outcome // weight
        )));
    };
    ($runtime:ident) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::DmpQueue($runtime::cumulus_pallet_dmp_queue::Event::ExecutedDownward {
                outcome,
                ..
            }) if outcome == &Outcome::Complete(50) // weight
        )));
    };
}

#[macro_export]
macro_rules! assert_withdrawal {
    ($runtime:ident, $who:expr, $amt:expr) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::Balances(pallet_balances::Event::Withdraw {
                who,
                amount,
            }) if who == &$who && amount == &$amt))
        );
    };
}

#[macro_export]
macro_rules! assert_asset_burned {
    ($runtime:ident, $id:expr, $who:expr, $amt:expr) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::Assets(pallet_assets::Event::Burned {
                asset_id,
                owner,
                balance,
            }) if asset_id == &$id && owner == &$who && balance == &$amt))
        );
    };
}

#[macro_export]
macro_rules! assert_asset_issued {
    ($runtime:ident, $id:expr, $who:expr, $amt:expr) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::Assets(pallet_assets::Event::Issued {
                asset_id,
                owner,
                amount,
            }) if asset_id == &$id && owner == &$who && amount == &$amt))
        );
    };
}

#[macro_export]
macro_rules! assert_deposit {
    ($runtime:ident, $who:expr, $amt:expr) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::Balances(pallet_balances::Event::Deposit {
                who,
                amount,
            }) if who == &$who && amount == &$amt))
        );
    };
    ($runtime:ident, $who:expr) => {
        assert!($runtime::System::events().iter().any(|r| matches!(
            &r.event,
            $runtime::RuntimeEvent::Balances(pallet_balances::Event::Deposit {
                who,
                ..
            }) if who == &$who))
        );
    };
}

#[macro_export]
macro_rules! teleport_from_relay_to {
    ($runtime:ident, $dest:expr, $beneficiary:expr, $amt:expr) => {
        assert_ok!($runtime::PolkadotXcm::send(
            $runtime::RuntimeOrigin::root(), // Act on behalf of this parachain sovereign
            box MultiLocation::parent().into_versioned(),
            box VersionedXcm::V3(Xcm(vec![
                Instruction::WithdrawAsset(MultiAssets::from(vec![MultiAsset {
                    id: AssetId::Concrete(MultiLocation::here()),
                    fun: Fungibility::Fungible($amt.into()),
                }])),
                BuyExecution {
                    fees: MultiAsset {
                        id: AssetId::Concrete(MultiLocation::here()),
                        fun: Fungibility::Fungible(60_000_000_000),
                    },
                    weight_limit: Limited(5_000_000_000.into()),
                },
                InitiateTeleport {
                    assets: MultiAssetFilter::Wild(All),
                    dest: $dest,
                    xcm: Xcm(vec![
                        BuyExecution {
                            fees: MultiAsset {
                                id: AssetId::Concrete(MultiLocation::parent()),
                                fun: Fungibility::Fungible(($amt / 10).into()),
                            },
                            weight_limit: Unlimited,
                        },
                        DepositAsset {
                            assets: Wild(All),
                            beneficiary: $beneficiary,
                        },
                        RefundSurplus
                    ]),
                },
                RefundSurplus
            ])),
        ));
        $runtime::System::events()
            .iter()
            .for_each(|r| println!(">>> [Large] {:?}", r.event));
        assert_polkadot_sent!($runtime);
    };
}
