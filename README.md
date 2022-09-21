### XBI Standard - interteam coworking dashboard

First implementation blueprint in `src/xbi_format.rs`

### Basic Types 
```rust
/// Global XBI Types.
/// Could also introduce t3rn-primitives/abi but perhaps easier to rely on sp_std / global types
pub type Data = Vec<u8>;
pub type AssetId = u32; // Could also be xcm::MultiAsset
pub type Value = u128; // Could also be [u64; 2] or sp_core::U128
pub type ValueEvm = sp_core::U256; // Could also be [u64; 4]
pub type Gas = u64; // [u64; 4]
pub type AccountId32 = sp_runtime::AccountId32;
pub type AccountId20 = sp_core::H160; // Could also take it from MultiLocation::Junction::AccountKey20 { network: NetworkId, key: [u8; 20] },

```

## XBI Format specification
```rust
pub enum XBIInstr {
  CallNative {
    payload: Data,
  },
  CallEvm {
    source: AccountId20, // Could use either [u8; 20] or Junction::AccountKey20
    dest: AccountId20,   // Could use either [u8; 20] or Junction::AccountKey20
    value: Value,
    input: Data,
    gas_limit: Gas,
    max_fee_per_gas: ValueEvm,
    max_priority_fee_per_gas: Option<ValueEvm>,
    nonce: Option<ValueEvm>,
    access_list: Vec<(AccountId20, Vec<sp_core::H256>)>, // Could use Vec<([u8; 20], Vec<[u8; 32]>)>,
  },
  CallWasm {
    dest: AccountId32,
    value: Value,
    gas_limit: Gas,
    storage_deposit_limit: Option<Value>,
    data: Data,
  },
  CallCustom {
    caller: AccountId32,
    dest: AccountId32,
    value: Value,
    input: Data,
    additional_params: Option<Vec<Data>>,
  },
  Transfer {
    dest: AccountId32,
    value: Value,
  },
  TransferORML {
    currency_id: AssetId,
    dest: AccountId32,
    value: Value,
  },
  TransferAssets {
    currency_id: AssetId,
    dest: AccountId32,
    value: Value,
  },
  Result {
    outcome: XBICheckOutStatus,
    output: Data,
    witness: Data,
  },
  Notification {
    kind: XBINotificationKind,
    instruction_id: Data,
    extra: Data,
  },
}
```

## XBI Metadata specification
- `metadata`: `Lifecycle status notifications`
    - `Sent (action timeout, notification timeout)`
    - `Delivered (action timeout, notification timeout)`
    - `Executed (action timeout, notification timeout)`
    - `Destination / Bridge security guarantees (e.g. in confirmation no for PoW, finality proofs)`
    - `max_exec_cost`: `Balance` : `Maximal cost / fees for execution of delivery`
    - `max_notification_cost`: `Balance` : `Maximal cost / fees per delivering notification`


Or directly from `src/xbi_format.rs`
```rust
#[derive(Clone, Eq, PartialEq, Debug, Default, Encode, Decode, TypeInfo)]
pub struct XBIMetadata {
  pub id: sp_core::H256,
  pub dest_para_id: u32,
  pub src_para_id: u32,
  pub sent: ActionNotificationTimeouts,
  pub delivered: ActionNotificationTimeouts,
  pub executed: ActionNotificationTimeouts,
  pub max_exec_cost: Value,
  pub max_notifications_cost: Value,
}
```