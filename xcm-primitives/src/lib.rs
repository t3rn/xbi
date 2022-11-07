#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use xcm::prelude::*;
use xcm::DoubleEncoded;

#[cfg(feature = "frame")]
pub mod frame_traits;

#[derive(Default)]
pub struct MultiLocationBuilder {
    inner: MultiLocation,
}

impl MultiLocationBuilder {
    pub fn get_relaychain_dest() -> VersionedMultiLocation {
        VersionedMultiLocation::V1(MultiLocationBuilder::new_native().with_parents(1).build())
    }

    pub fn new_native() -> Self {
        Self {
            inner: MultiLocation {
                parents: 0,
                interior: Here,
            },
        }
    }

    pub fn new_parachain(parachain: u32) -> Self {
        Self {
            inner: MultiLocation {
                parents: 0,
                interior: X1(Parachain(parachain)),
            },
        }
    }

    pub fn new_account_32(parent: Option<u8>, account: [u8; 32]) -> Self {
        Self {
            inner: MultiLocation {
                parents: parent.unwrap_or_default(),
                interior: X1(AccountId32 {
                    network: Any,
                    id: account,
                }),
            },
        }
    }

    pub fn with_parents(mut self, parents: u8) -> Self {
        self.inner.parents = parents;
        self
    }

    pub fn with_junction(mut self, jnc: Junction) -> Self {
        match self.inner.interior {
            // Overwrite the last action
            X8(t, u, v, w, x, y, z, _a) => {
                self.inner.interior = X8(t, u, v, w, x, y, z, jnc);
            }
            _ => {
                // We handle the overflow above
                let _ = self.inner.push_interior(jnc);
            }
        }
        self
    }

    pub fn build(self) -> MultiLocation {
        self.inner
    }
}
pub struct XcmBuilder<T> {
    inner: Xcm<T>,
}

impl<T> Default for XcmBuilder<T> {
    fn default() -> Self {
        Self { inner: Xcm::new() }
    }
}

impl<T: Codec> XcmBuilder<T> {
    pub fn with_transfer_self_reserve(
        mut self,
        assets: MultiAssets,
        fee: u128,
        dest: MultiLocation,
        recipient: MultiLocation,
        weight_limit: Option<u64>,
    ) -> Self {
        let reserve_xcm = XcmBuilder::default()
            .with_buy_execution(dest.clone(), fee, weight_limit.map(WeightLimit::Limited))
            .with_deposit_asset(recipient, assets.len() as u32)
            .build();
        self.inner.0.push(TransferReserveAsset {
            assets,
            dest,
            // This is injected and called by the dest (self)
            xcm: reserve_xcm,
        });
        self
    }

    pub fn with_transfer_reserve_to_reserve(
        mut self,
        assets: MultiAssets,
        fee: u128,
        reserve: MultiLocation,
        recipient: MultiLocation,
        weight_limit: Option<u64>,
    ) -> Self {
        let injected_xcm = XcmBuilder::default()
            .with_buy_execution(reserve.clone(), fee, weight_limit.map(WeightLimit::Limited))
            .with_deposit_asset(recipient, assets.len() as u32)
            .build();
        self.inner.0.push(InitiateReserveWithdraw {
            assets: Wild(All),
            reserve,
            // This is injected and called by the reserve(target)
            xcm: injected_xcm,
        });
        self
    }

    // Simply a large function
    #[allow(clippy::too_many_arguments)]
    pub fn with_transfer(
        mut self,
        assets: MultiAssets,
        execution_fee: u128,
        reserve: MultiLocation,
        dest: MultiLocation,
        recipient: MultiLocation,
        weight_limit: Option<u64>,
        // Whether the reserve can teleport the transfer
        should_teleport: bool,
    ) -> Self {
        let mut reanchored_dest = dest.clone();
        if reserve == MultiLocation::parent() {
            match dest {
                MultiLocation {
                    parents,
                    interior: X1(Parachain(id)),
                } if parents == 1 => {
                    reanchored_dest = Parachain(id).into();
                }
                _ => {}
            }
        }

        self.inner.0.push(WithdrawAsset(assets.clone()));

        self.inner.0.push(InitiateReserveWithdraw {
            assets: Wild(All),
            reserve: reserve.clone(),
            xcm: if should_teleport {
                XcmBuilder::default()
                    .with_buy_execution(reserve, execution_fee / 2, weight_limit.map(Limited))
                    .with_initiate_teleport(
                        reanchored_dest,
                        recipient,
                        execution_fee / 2,
                        weight_limit,
                        assets,
                    )
                    .build()
            } else {
                XcmBuilder::default()
                    .with_buy_execution(reserve, execution_fee / 2, weight_limit.map(Limited))
                    .with_deposit_reserve_asset(
                        reanchored_dest,
                        recipient,
                        execution_fee / 2,
                        weight_limit,
                        assets,
                    )
                    .build()
            },
        });
        self
    }

    pub fn with_initiate_teleport(
        mut self,
        dest: MultiLocation,
        recipient: MultiLocation,
        execution_fee: u128,
        weight_limit: Option<u64>,
        assets: MultiAssets,
    ) -> XcmBuilder<T> {
        self.inner.0.push(InitiateTeleport {
            assets: Wild(All), // TODO: this needs fixing
            dest: dest.clone(),
            xcm: XcmBuilder::default()
                .with_buy_execution(dest, execution_fee, weight_limit.map(WeightLimit::Limited))
                .with_deposit_asset(recipient, assets.len() as u32)
                .build(),
        });
        self
    }

    pub fn with_deposit_reserve_asset(
        mut self,
        dest: MultiLocation,
        recipient: MultiLocation,
        execution_fee: u128,
        weight_limit: Option<u64>,
        assets: MultiAssets,
    ) -> XcmBuilder<T> {
        self.inner.0.push(DepositReserveAsset {
            assets: Wild(All),
            max_assets: assets.len() as u32,
            dest: dest.clone(),
            xcm: XcmBuilder::default()
                .with_buy_execution(dest, execution_fee, weight_limit.map(WeightLimit::Limited))
                .with_deposit_asset(recipient, assets.len() as u32)
                .build(),
        });
        self
    }

    pub fn with_withdraw_concrete_asset(
        mut self,
        asset: MultiLocation,
        amt: u128,
    ) -> XcmBuilder<T> {
        self.inner
            .0
            .push(WithdrawAsset(MultiAssets::from(vec![MultiAsset {
                id: Concrete(asset),
                fun: Fungible(amt),
            }])));
        self
    }

    pub fn with_buy_execution(
        mut self,
        asset: MultiLocation,
        amt: u128,
        weight_limit: Option<WeightLimit>,
    ) -> XcmBuilder<T> {
        self.inner.0.push(BuyExecution {
            fees: MultiAsset {
                id: Concrete(asset),
                fun: Fungible(amt),
            },
            weight_limit: weight_limit.unwrap_or(WeightLimit::Unlimited),
        });
        self
    }

    pub fn with_transact(
        mut self,
        origin_type: Option<OriginKind>,
        max_weight: Option<u64>,
        call: Vec<u8>,
    ) -> XcmBuilder<T> {
        let call: DoubleEncoded<T> = call.into();
        self.inner.0.push(Transact {
            origin_type: origin_type.unwrap_or(OriginKind::Native),
            require_weight_at_most: max_weight.unwrap_or(1_000_000_000),
            call,
        });
        self
    }

    pub fn with_deposit_asset(
        mut self,
        beneficiary: MultiLocation,
        max_assets: u32,
    ) -> XcmBuilder<T> {
        self.inner.0.push(DepositAsset {
            assets: Wild(All),
            max_assets,
            beneficiary,
        });
        self
    }

    pub fn with_refund_surplus(mut self) -> XcmBuilder<T> {
        self.inner.0.push(RefundSurplus);
        self
    }

    pub fn build(self) -> Xcm<T> {
        self.inner
    }
}
