use super::*;
use frame_support::{parameter_types, traits::AsEnsureOriginWithArg};
use frame_system::EnsureRoot;
use sp_core::ConstU32;

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
    type AssetIdParameter = AssetId;
    type Balance = Balance;
    type CallbackHandle = ();
    type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
    type Currency = Balances;
    type Extra = ();
    type ForceOrigin = EnsureRoot<AccountId>;
    type Freezer = ();
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type RemoveItemsLimit = ConstU32<1>;
    type RuntimeEvent = RuntimeEvent;
    type StringLimit = AssetsStringLimit;
    type WeightInfo = ();
}

parameter_types! {
    pub const RegCost: u128 = 100_000_000_000;
}

impl pallet_asset_registry::Config for Runtime {
    type Assets = Assets;
    type Currency = Balances;
    type RegistrationCost = RegCost;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
}
