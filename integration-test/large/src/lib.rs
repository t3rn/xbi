#![feature(box_syntax)]

use cumulus_parachains_common::{
    AuraId, MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO, SLOT_DURATION,
};
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU8, Everything, OnTimestampSet},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight},
        ConstantMultiplier, DispatchClass, IdentityFee,
    },
};
use frame_system::limits::{BlockLength, BlockWeights};
use polkadot_primitives::v2::{Id as ParaId, Moment};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, AccountId32, Perbill};
use xcm::latest::prelude::*;

pub use cumulus_pallet_dmp_queue;
pub use cumulus_pallet_xcmp_queue;

pub mod assets_config;
pub mod contracts_config;
pub mod xcm_config;

pub type AccountId = AccountId32;
pub type Balance = u128;
pub type Amount = i128;

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
        // This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
    //  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
    // `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
    // the lazy contract deletion.
    pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have some extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();
}

impl frame_system::Config for Runtime {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountId;
    type BaseCallFilter = Everything;
    type BlockHashCount = BlockHashCount;
    type BlockLength = ();
    type BlockNumber = u64;
    type BlockWeights = ();
    type RuntimeCall = RuntimeCall;
    type DbWeight = ();
    type RuntimeEvent = RuntimeEvent;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type Header = Header;
    type Index = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    type RuntimeOrigin = RuntimeOrigin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

pub struct MockOnTimestampSet;
impl OnTimestampSet<Moment> for MockOnTimestampSet {
    fn on_timestamp_set(_moment: Moment) {
        // Do nothing
    }
}

impl pallet_timestamp::Config for Runtime {
    type MinimumPeriod = MinimumPeriod;
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    // type OnTimestampSet = Aura;
    type OnTimestampSet = MockOnTimestampSet;
    type WeightInfo = ();
}

parameter_types! {
    pub const TransactionByteFee: Balance = 1;
    pub const MaxAuthorities: u32 = 100_000;

}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type FeeMultiplierUpdate = ();
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = IdentityFee<Balance>;
}

parameter_types! {
    pub ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = MaxAuthorities;
}

impl t3rn_primitives::EscrowTrait<Runtime> for Runtime {
    type Currency = Balances;
    type Time = Timestamp;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl parachain_info::Config for Runtime {}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Config, Event<T>},
        ParachainInfo: parachain_info::{Pallet, Storage, Config},
        XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>},
        DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>},
        CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin},
        PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
        Timestamp: pallet_timestamp,
        TransactionPayment: pallet_transaction_payment,
        Evm: pallet_3vm_evm,
        RandomnessCollectiveFlip: pallet_randomness_collective_flip,
        // ThreeVm: pallet_3vm,
        Contracts: pallet_3vm_contracts,
        Aura: pallet_aura,
        Assets: pallet_assets = 20,
        AssetRegistry: pallet_asset_registry = 21,
        XbiPortal: pallet_xbi_portal = 200
    }
);
