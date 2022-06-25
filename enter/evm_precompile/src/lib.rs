// SPDX-License-Identifier: Apache-2.0
// This file is part of Frontier.
//
// Copyright (c) 2020-2022 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use core::marker::PhantomData;
use fp_evm::{
    Context, ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileOutput,
    PrecompileResult,
};
use frame_support::{
    codec::Decode,
    dispatch::{Dispatchable, Encode, GetDispatchInfo, PostDispatchInfo},
    weights::{DispatchClass, Pays},
};
use pallet_evm::{AddressMapping, GasWeightMapping};
use pallet_xbi_portal::xbi_codec::XBIFormat;

pub struct Dispatch<T> {
    _marker: PhantomData<T>,
}

impl<T> Precompile for Dispatch<T>
where
    T: frame_system::Config + pallet_evm::Config + pallet_xbi_portal::Config,
    <T as frame_system::Config>::Call:
        Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
    <<T as frame_system::Config>::Call as Dispatchable>::Origin: From<Option<T::AccountId>>,
{
    fn execute(
        input: &[u8],
        target_gas: Option<u64>,
        context: &Context,
        _is_static: bool,
    ) -> PrecompileResult {
        // Assume input is encoded XBIFormat message: includes both instruction and metadata
        let xbi: XBIFormat =
            Decode::decode(&mut &input[..]).map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("decode XBI Format failed".into()),
            })?;

        let call_xbi = pallet_xbi_portal::Call::check_in_xbi::<T> { xbi };

        let call = <T as frame_system::Config>::Call::decode(&mut &call_xbi.encode()[..]).map_err(
            |_| PrecompileFailure::Error {
                exit_status: ExitError::Other("decode failed".into()),
            },
        )?;

        let info = call.get_dispatch_info();

        let valid_call = info.pays_fee == Pays::Yes && info.class == DispatchClass::Normal;
        if !valid_call {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("invalid call".into()),
            })
        }

        if let Some(gas) = target_gas {
            let valid_weight = info.weight <= T::GasWeightMapping::gas_to_weight(gas);
            if !valid_weight {
                return Err(PrecompileFailure::Error {
                    exit_status: ExitError::OutOfGas,
                })
            }
        }

        let origin = T::AddressMapping::into_account_id(context.caller);

        match call.dispatch(Some(origin).into()) {
            Ok(post_info) => {
                let cost = T::GasWeightMapping::weight_to_gas(
                    post_info.actual_weight.unwrap_or(info.weight),
                );
                Ok(PrecompileOutput {
                    exit_status: ExitSucceed::Stopped,
                    cost,
                    output: Default::default(),
                    logs: Default::default(),
                })
            },
            Err(_) => Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("dispatch execution failed".into()),
            }),
        }
    }
}
