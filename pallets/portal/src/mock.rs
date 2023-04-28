use crate as pallet_xbi_portal;
use frame_support::{
    parameter_types,
    traits::{ConstU16, ConstU64},
    weights::IdentityFee,
};
use frame_system as system;
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::ConstU32,
    traits::{BlakeTwo256, IdentityLookup},
};

pub type Balance = u128;
pub type AssetId = u32;
pub type AccountId = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        XbiPortal: pallet_xbi_portal,
        Assets: pallet_assets,
        Balances: pallet_balances,
    }
);

impl system::Config for Test {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountId;
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockHashCount = ConstU64<250>;
    type BlockLength = ();
    type BlockNumber = u64;
    type BlockWeights = ();
    type RuntimeCall = RuntimeCall;
    type DbWeight = ();
    type RuntimeEvent = RuntimeEvent;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type RuntimeOrigin = RuntimeOrigin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ConstU16<42>;
    type SystemWeightInfo = ();
    type Version = ();
}

parameter_types! {
    pub XcmSovereignOrigin: u64 = 5;
}

pub struct NonsenseNoopEvm;
impl evm_primitives::traits::Evm<<Test as frame_system::Config>::Origin> for NonsenseNoopEvm {
    type Outcome = Result<
        (
            evm_primitives::CallInfo,
            frame_support::pallet_prelude::Weight,
        ),
        sp_runtime::DispatchError,
    >;
    fn call(
        _origin: <Test as frame_system::Config>::Origin,
        _source: sp_core::H160,
        _target: sp_core::H160,
        _input: Vec<u8>,
        _value: sp_core::U256,
        _gas_limit: u64,
        _max_fee_per_gas: sp_core::U256,
        _max_priority_fee_per_gas: Option<sp_core::U256>,
        _nonce: Option<sp_core::U256>,
        _access_list: Vec<(sp_core::H160, Vec<sp_core::H256>)>,
    ) -> Self::Outcome {
        Ok((
            evm_primitives::CallInfo {
                exit_reason: evm_primitives::ExitReason::Succeed(
                    evm_primitives::ExitSucceed::Stopped,
                ),
                value: vec![],
                used_gas: Default::default(),
                logs: vec![],
            },
            0,
        ))
    }
}

parameter_types! {
    pub ReserveBalanceCustodian: AccountId = 64;
}

impl pallet_xbi_portal::Config for Test {
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type XcmSovereignOrigin = XcmSovereignOrigin;
    type Xcm = ();
    type Contracts = ();
    type Evm = NonsenseNoopEvm;
    type Currency = Balances;
    type AssetRegistry = ();
    type Callback = ();
    type CheckInLimit = ConstU32<100>;
    type CheckInterval = ConstU64<3>;
    type CheckOutLimit = ConstU32<100>;
    type ExpectedBlockTimeMs = ConstU32<6000>;
    type ParachainId = ConstU32<3333>;
    type TimeoutChecksLimit = ConstU32<3000>;
    type Assets = Assets;
    type FeeConversion = IdentityFee<Balance>;
    type DeFi = ();
    type ReserveBalanceCustodian = ReserveBalanceCustodian;
    type NotificationWeight = ConstU64<1>;
}

parameter_types! {
    pub ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
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

parameter_types! {
    pub const AssetDeposit: Balance = 1; // 1 UNIT deposit to create asset
    pub const ApprovalDeposit: Balance = 1;
    pub const AssetsStringLimit: u32 = 50;
    /// Key = 32 bytes, Value = 36 bytes (32+1+1+1+1)
    // https://github.com/paritytech/substrate/blob/069917b/frame/assets/src/lib.rs#L257L271
    pub const MetadataDepositBase: Balance = 0;
    pub const MetadataDepositPerByte: Balance = 0;
    pub const AssetAccountDeposit: Balance = 0;
}

impl pallet_assets::Config for Test {
    type ApprovalDeposit = ApprovalDeposit;
    type AssetAccountDeposit = AssetAccountDeposit;
    type AssetDeposit = AssetDeposit;
    type AssetId = AssetId;
    type Balance = Balance;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type Extra = ();
    type ForceOrigin = EnsureRoot<Self::AccountId>;
    type Freezer = ();
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type StringLimit = AssetsStringLimit;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    sp_io::TestExternalities::default()
}
