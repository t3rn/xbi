use super::*;
use frame_support::parameter_types;
use frame_system::EnsureRoot;

pub type AssetId = u32;

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

impl pallet_assets::Config for Runtime {
    type ApprovalDeposit = ApprovalDeposit;
    type AssetAccountDeposit = AssetAccountDeposit;
    type AssetDeposit = AssetDeposit;
    type AssetId = AssetId;
    type Balance = Balance;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type Extra = ();
    type ForceOrigin = EnsureRoot<AccountId>;
    type Freezer = ();
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type StringLimit = AssetsStringLimit;
    type WeightInfo = ();
}

parameter_types! {
    pub const RegCost: u128 = 100_000_000_000;
}

impl pallet_asset_registry::Config for Runtime {
    type Assets = Assets;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type RegistrationCost = RegCost;
}
