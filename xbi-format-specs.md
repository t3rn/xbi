### XBI Standard - interteam coworking dashboard

First implementation blueprint in `src/xbi_format.rs`

### Basic Types 
```rust
/// Basic Types.
/// Could also introduce t3rn-primitives/abi but perhaps easier to align on sp_std types
pub type Bytes = Vec<u8>;
/// Introduce enum vs u32/u64 and cast later?
pub type AssetId = u64;
/// Could be a MultiAsset?
pub type Balance16B = u128;
pub type AccountIdOf = MultiLocation;
```

## XBI Format specification
```rust
    CallNative {
        payload: Bytes, // assumes to be encoded Call that can be dispatched
    },
    CallModule { // call modules like FRAME pallets by pallet_id + method_id
        module_id: u8, 
        method_id: u8, 
    },
    CallEvm {
        caller: AccountId32,
        dest: Junction, // Junction::AccountKey20
        value: Balance16B,
        input: Bytes,
        gas_limit: Balance16B,
        max_fee_per_gas: Option<Balance16B>,
        max_priority_fee_per_gas: Option<Balance16B>,
        nonce: Option<u32>,
        access_list: Option<Bytes>,
    },
    CallWasm {
        caller: AccountId32,
        dest: AccountId32,
        value: Balance16B,
        input: Bytes,
    },
    CallCustom {
        caller: AccountId32,
        dest: AccountId32,
        value: Balance16B,
        input: Bytes,
        additional_params: Option<Vec<Bytes>>,
    },
    Transfer {
        dest: AccountId32,
        value: Balance16B,
    },
    TransferORML {
        currency_id: AssetId,
        dest: AccountId32,
        value: Balance16B,
    },
    TransferAssets {
        currency_id: AssetId,
        dest: AccountId32,
        value: Balance16B,
    },
    Result {
        outcome: XBICheckOutStatus,
        output: Bytes,
        witness: Bytes,
    },
    Notification {
        kind: XBINotificationKind,
        instruction_id: Bytes,
        extra: Bytes,
    },
},
```

## XBI Metadata specification
- `metadata`: `Lifecycle status notifications`
    - `Sent (action timeout, notification timeout)`
    - `Delivered (action timeout, notification timeout)`
    - `Executed (action timeout, notification timeout)`
    - `Destination / Bridge security guarantees (e.g. in confirmation no for PoW, finality proofs)`
    - `max_exec_cost`: `Balance` : `Maximal cost / fees for execution of delivery`
    - `max_notification_cost`: `Balance` : `Maximal cost / fees per delivering notification`

```rust
#[derive(Clone, Eq, PartialEq, Debug, Default, Encode, Decode, TypeInfo)]
pub struct XBIMetadata {
    pub id: sp_core::H256,
    pub dest_para_id: u32,
    pub src_para_id: u32,
    pub sent: ActionNotificationTimeouts,
    pub delivered: ActionNotificationTimeouts,
    pub executed: ActionNotificationTimeouts,
    pub max_exec_cost: Balance16B,
    pub max_notifications_cost: Balance16B,
}
```