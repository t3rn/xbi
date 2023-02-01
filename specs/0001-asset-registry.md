# Asset registry for XBI

There is a need for a simplistic way to route assets which may be configured by anyone, into assets that we know about.

Ultimately, we want parachains and external actors to be able to inject liquidity to t3rn and use it to pay for XBI operations.

## Technical information

### Lookups
We require some way of routing:
- an arbitrary MultiLocation to a PalletAssets(id) 
- a PalletAssets(id) to an arbitrary MultiLocation

These requirements can be easily seen in the `AssetTransactors` in `integration_test::large::xcm_config`. This can be done one of two ways:
- algorithmically, lots of corner cases, duplicates and difficult to get right
- some storage, nice and simple

The lookup functionality(`multilocation -> id`) is a many to one relationship, with the multilocation being configurable by the location itself, or an owner of the asset representation.

E.g: a parachain may want to update this index key via XCM, so will send an XBI message to us via `OriginKind::SovereignAccount`, which we know is the parachain and can safely update itself.
Otherwise, some owner of the representation can also introduce changes to the mapping.

Anyone should be able to introduce new lookups for their applications.
E.g some developer outside of Basilisk might want some nice mapping which is simply `Multilocation { parents: 0, junctions: Junction::X1(Junction::GeneralIndex(505505050534)) }` -> `AssetId(BasiliskAssetId)`

We should, however, reserve parachain junctions for parachains, so this should only be sent via XBI.

This would require some lookup trait added to the pallet which we can use instead of `integration_test::large::xcm_config::RelayToInternalAssetId` or `AsPrefixedGeneralIndex` which algorithmically determines the lookups.

### Asset Capabilities

We also need some capabilities field that we can bolster to our representation of an asset, such as:
- can teleport? who is the checking account if any? this would affect `TrustedTeleporters`
- is a reserve? TODO: more readup on reserves
- can pay for fees? would require guards in the channels to protect from execution for these assets
- what is the cost per weight for transacting on this asset? this solves an issue where anyone can just mint an asset with 127 decimals and have cheap xcm fees
        This would affect `Weighers`, `Trader`

We should try to add these capabilities for blocks and such to `Barrier`

Naturally, these cannot be configured by anyone, and would normally have external mechanisms such as liquidity pools and such to determine the cost per weight. For now we will set them when we onboard new parachains to XBI.