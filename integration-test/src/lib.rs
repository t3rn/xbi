#![feature(box_syntax)]

use bytes::buf::Buf;
use frame_support::{assert_ok, pallet_prelude::Weight, traits::GenesisBuild};
use hex::ToHex;
use log::LevelFilter;
use pallet_asset_registry::VersionedMultiAssets;
pub use rococo_runtime as rococo;
use sp_runtime::AccountId32;
use xcm::{
    latest::prelude::*,
    v1::{Junction, Junctions, MultiLocation},
};
use xcm_emulator::{decl_test_network, decl_test_relay_chain, TestExt};

use crate::{
    large::LARGE_PARA_ID,
    slim::{SLENDER_PARA_ID, SLIM_PARA_ID},
};

mod large;
pub mod macros;
mod slim;

pub const ALICE: AccountId32 = AccountId32::new([0u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([1u8; 32]);
pub const CONTRACT_CALLER: AccountId32 = AccountId32::new([55u8; 32]);
pub const INITIAL_BALANCE: u128 = 1_000_000_000_000_000;

// 6d6f646c70792f78636d63680000000000000000000000000000000000000000
// 5EYCAe5ijiYgWYWi1fs8Xz1td1djEtJVVnNfzvDRP4VtLL7Y
pub const ROCOCO_CHECKIN_ACCOUNT: AccountId32 = AccountId32::new([
    109, 111, 100, 108, 112, 121, 47, 120, 99, 109, 99, 104, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0,
]);

#[derive(Debug, Clone)]
pub enum ParaKind {
    Sibling(u32),
    Child(u32),
}

// TODO: move this into xcm-primitives
pub fn para_id_to_account(para: ParaKind) -> AccountId32 {
    let mut bytes = [0u8; 32];
    match para {
        ParaKind::Child(id) => {
            let para_bytes: String = b"para".encode_hex();
            let para_bytes = hex::decode(para_bytes).unwrap();
            para_bytes.as_slice().copy_to_slice(&mut bytes[0..4]);
            id.to_le_bytes().as_slice().copy_to_slice(&mut bytes[4..8]);
        }
        ParaKind::Sibling(id) => {
            let sibl_bytes: String = b"sibl".encode_hex();
            let sibl_bytes = hex::decode(sibl_bytes).unwrap();
            sibl_bytes.as_slice().copy_to_slice(&mut bytes[0..4]);
            id.to_le_bytes().as_slice().copy_to_slice(&mut bytes[4..8]);
        }
    }
    let acc = AccountId32::new(bytes);

    log::info!("Converted {:?} into AccountId32 {:?}", para, acc);
    acc
}

fn log_all_roco_events() {
    rococo::System::events()
        .iter()
        .for_each(|r| println!(">>> [Rococo] {:?}", r.event));
}

pub fn force_xcm_version(para: u32, version: u32) {
    RococoNet::execute_with(|| {
        assert_ok!(rococo::XcmPallet::force_xcm_version(
            rococo::Origin::root(),
            box MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::Parachain(para)),
            },
            version,
        ));
        log_all_roco_events();
        rococo::System::reset_events();
    });
}

pub fn force_default_xcm_version(version: u32) {
    RococoNet::execute_with(|| {
        assert_ok!(rococo::XcmPallet::force_default_xcm_version(
            rococo::Origin::signed(ALICE),
            Some(version),
        ));
        log_all_roco_events();
        rococo::System::reset_events();
    });
}

pub fn transfer_to(dest: AccountId32, amt: u128) {
    RococoNet::execute_with(|| {
        assert_ok!(rococo::Balances::transfer(
            rococo::Origin::signed(ALICE),
            sp_runtime::MultiAddress::Id(dest.clone()),
            amt,
        ));
        log_all_roco_events();
        assert!(rococo::System::events().iter().any(|r| matches!(
            &r.event,
            rococo::Event::Balances(pallet_balances::Event::Transfer {
                from: ALICE,
                to,
                amount
            }) if to == &dest && amount == &amt
        )));
        rococo::System::reset_events();
    });
}

// Note, this is a little buggy, better use the macro instead
pub fn teleport_to(dest: MultiLocation, beneficiary: MultiLocation, amount: u128) {
    RococoNet::execute_with(|| {
        let assets = MultiAssets::from(vec![MultiAsset {
            id: AssetId::Concrete(MultiLocation::here()),
            fun: Fungibility::Fungible(amount),
        }]);
        let assets = VersionedMultiAssets::V1(assets);
        log::info!("Teleporting {:?} to {:?}", assets, dest);
        assert_ok!(rococo::XcmPallet::teleport_assets(
            rococo::Origin::signed(ALICE),
            box dest.into(),
            box beneficiary.into(),
            box assets,
            0,
        ));
        log_all_roco_events();
        rococo::System::reset_events();
    });
}

pub fn setup() {
    Network::reset();
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Off)
        // .with_module_level("asset-registry", LevelFilter::Debug)
        // .with_module_level("xcm", LevelFilter::Debug)
        .with_module_level("xbi", LevelFilter::Debug)
        .with_module_level("xp-channel", LevelFilter::Debug)
        .with_module_level("frame-receiver", LevelFilter::Debug)
        .with_module_level("trie", LevelFilter::Off)
        .init()
        .ok();

    force_xcm_version(SLIM_PARA_ID, 2);
    force_xcm_version(SLENDER_PARA_ID, 2);
    force_xcm_version(LARGE_PARA_ID, 2);
}

decl_test_relay_chain! {
    pub struct RococoNet {
        Runtime = rococo_runtime::Runtime,
        XcmConfig = rococo_runtime::xcm_config::XcmConfig,
        new_ext = rococo_ext(),
    }
}

decl_test_network! {
    pub struct Network {
        relay_chain = RococoNet,
        parachains = vec![
            (1, slim::Slim),
            (2, slim::Slender),
            (3, large::Large),
        ],
    }
}

pub fn rococo_ext() -> sp_io::TestExternalities {
    use rococo_runtime::{Runtime, System};

    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![(ALICE, INITIAL_BALANCE)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    polkadot_runtime_parachains::configuration::GenesisConfig::<Runtime> {
        config: default_parachains_host_configuration(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn default_parachains_host_configuration(
) -> polkadot_runtime_parachains::configuration::HostConfiguration<
    polkadot_primitives::v2::BlockNumber,
> {
    use polkadot_primitives::v2::{MAX_CODE_SIZE, MAX_POV_SIZE};

    polkadot_runtime_parachains::configuration::HostConfiguration {
        minimum_validation_upgrade_delay: 5,
        validation_upgrade_cooldown: 10u32,
        validation_upgrade_delay: 10,
        code_retention_period: 1200,
        max_code_size: MAX_CODE_SIZE,
        max_pov_size: MAX_POV_SIZE,
        max_head_data_size: 32 * 1024,
        group_rotation_frequency: 20,
        chain_availability_period: 4,
        thread_availability_period: 4,
        max_upward_queue_count: 8,
        max_upward_queue_size: 1024 * 1024,
        max_downward_message_size: 1024,
        ump_service_total_weight: Weight::from(4_u32 * 1_000_000_000_u32),
        max_upward_message_size: 50 * 1024,
        max_upward_message_num_per_candidate: 5,
        hrmp_sender_deposit: 0,
        hrmp_recipient_deposit: 0,
        hrmp_channel_max_capacity: 8,
        hrmp_channel_max_total_size: 8 * 1024,
        hrmp_max_parachain_inbound_channels: 4,
        hrmp_max_parathread_inbound_channels: 4,
        hrmp_channel_max_message_size: 1024 * 1024,
        hrmp_max_parachain_outbound_channels: 4,
        hrmp_max_parathread_outbound_channels: 4,
        hrmp_max_message_num_per_candidate: 5,
        dispute_period: 6,
        no_show_slots: 2,
        n_delay_tranches: 25,
        needed_approvals: 2,
        relay_vrf_modulo_samples: 2,
        zeroth_delay_tranche_width: 0,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sovereign_calculator() {
        assert_eq!(
            para_id_to_account(ParaKind::Child(2000)),
            AccountId32::new([
                112, 97, 114, 97, 208, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0
            ])
        );
    }
    #[test]
    fn test_sibl_sovereign_calculator() {
        assert_eq!(
            para_id_to_account(ParaKind::Sibling(3)),
            AccountId32::new([
                115, 105, 98, 108, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0
            ])
        );
    }
}
