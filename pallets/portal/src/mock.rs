use crate as pallet_xbi_portal;
use frame_support::{
    parameter_types,
    traits::{AsEnsureOriginWithArg, ConstU16, ConstU64},
    weights::{IdentityFee, Weight},
};
use frame_system as system;
use frame_system::EnsureRoot;
use pallet_evm::ExitSucceed;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, ConstU32, IdentityLookup},
};

use pallet_evm::ExitReason;
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
        System: frame_system,
        XbiPortal: pallet_xbi_portal,
        Assets: pallet_assets,
        Balances: pallet_balances,
    }
);

impl system::Config for Test {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountId;
    type BaseCallFilter = frame_support::traits::Everything;
    type Block = Block;
    type BlockHashCount = ConstU64<250>;
    type BlockLength = ();
    type BlockWeights = ();
    type DbWeight = ();
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type Nonce = u32;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type PalletInfo = PalletInfo;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type SS58Prefix = ConstU16<42>;
    type SystemWeightInfo = ();
    type Version = ();
}

parameter_types! {
    pub XcmSovereignOrigin: u64 = 5;
}

pub struct NonsenseNoopEvm;
impl t3rn_primitives::threevm::Evm<<Test as frame_system::Config>::RuntimeOrigin>
    for NonsenseNoopEvm
{
    type Outcome = Result<crate::HandlerInfo<Weight>, sp_runtime::DispatchError>;

    fn call(
        _origin: <Test as frame_system::Config>::RuntimeOrigin,
        _target: sp_core::H160,
        _input: Vec<u8>,
        _value: sp_core::U256,
        _gas_limit: u64,
        _max_fee_per_gas: sp_core::U256,
        _max_priority_fee_per_gas: Option<sp_core::U256>,
        _nonce: Option<sp_core::U256>,
        _access_list: Vec<(sp_core::H160, Vec<sp_core::H256>)>,
    ) -> Self::Outcome {
        Err(sp_runtime::DispatchError::Other(
            "NonsenseNoopEvm not implemented",
        ))
    }
}

use contracts_primitives::{ContractExecResult, ExecReturnValue, ReturnFlags, StorageDeposit};
use frame_system::EventRecord;
pub struct NonsenseNoopWasm;
impl t3rn_primitives::threevm::Contracts<u64, u128, EventRecord<RuntimeEvent, sp_core::H256>>
    for NonsenseNoopWasm
{
    type Outcome = ContractExecResult<u128, EventRecord<RuntimeEvent, sp_core::H256>>;

    fn call(
        _origin: AccountId,
        _dest: AccountId,
        _value: Balance,
        _gas_limit: Weight,
        _storage_deposit_limit: Option<Balance>,
        _data: Vec<u8>,
        _debug: bool,
    ) -> Self::Outcome {
        ContractExecResult::<Balance, EventRecord<RuntimeEvent, sp_core::H256>> {
            gas_consumed: Weight::zero(),
            gas_required: Weight::zero(),
            debug_message: Vec::new(),
            storage_deposit: StorageDeposit::Refund(Default::default()),
            result: Ok(ExecReturnValue {
                flags: ReturnFlags::empty(),
                data: Vec::default(),
            }),
            events: None,
        }
    }
}

parameter_types! {
    pub ReserveBalanceCustodian: AccountId = 64;
    pub NotificationWeight: Weight = Weight::from_parts(1, 0u64);
}

impl pallet_xbi_portal::Config for Test {
    type AssetRegistry = ();
    type Assets = Assets;
    type Callback = ();
    type CheckInLimit = ConstU32<100>;
    type CheckInterval = ConstU64<3>;
    type CheckOutLimit = ConstU32<100>;
    type Contracts = NonsenseNoopWasm;
    type Currency = Balances;
    type DeFi = ();
    type Evm = NonsenseNoopEvm;
    type ExpectedBlockTimeMs = ConstU32<6000>;
    type FeeConversion = IdentityFee<Balance>;
    type NotificationWeight = NotificationWeight;
    type ParachainId = ConstU32<3333>;
    type ReserveBalanceCustodian = ReserveBalanceCustodian;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type TimeoutChecksLimit = ConstU32<3000>;
    type Xcm = ();
    type XcmSovereignOrigin = XcmSovereignOrigin;
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
    type ExistentialDeposit = ExistentialDeposit;
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type MaxHolds = ();
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type RuntimeEvent = RuntimeEvent;
    type RuntimeHoldReason = ();
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
    type AssetIdParameter = AssetId;
    type Balance = Balance;
    type CallbackHandle = ();
    type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
    type Currency = Balances;
    type Extra = ();
    type ForceOrigin = EnsureRoot<Self::AccountId>;
    type Freezer = ();
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type RemoveItemsLimit = ConstU32<1>;
    type RuntimeEvent = RuntimeEvent;
    type StringLimit = AssetsStringLimit;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    sp_io::TestExternalities::default()
}
