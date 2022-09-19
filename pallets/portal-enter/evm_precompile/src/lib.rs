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
    ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
    PrecompileResult,
};
use frame_support::dispatch::RawOrigin;
use frame_support::dispatch::UnfilteredDispatchable;
use frame_support::{
    codec::Decode,
    dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
};




use xbi_format::xbi_codec::{XBIFormat, XBIInstr, XBIMetadata};

pub struct XBIPortal<T> {
    _marker: PhantomData<T>,
}

pub fn xbi_metadata_origin_2_local_account<T: frame_system::Config>(
    metadata: &XBIMetadata,
) -> Result<T::AccountId, PrecompileFailure> {
    let from = metadata
        .maybe_known_origin
        .as_ref()
        .ok_or(PrecompileFailure::Error {
            exit_status: ExitError::Other("dispatch execution failed".into()),
        })?;

    sabi::associate(from.clone()).map_err(|_e| PrecompileFailure::Error {
        exit_status: ExitError::Other("dispatch execution failed".into()),
    })
}

pub fn custom_decode_xbi<T: frame_system::Config>(
    input: Vec<u8>,
) -> Result<XBIFormat, PrecompileFailure> {
    Decode::decode(&mut &input[..]).map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("decode XBI Format failed".into()),
    })
}

impl<T> Precompile for XBIPortal<T>
where
    T: frame_system::Config + pallet_evm::Config + pallet_balances::Config,
    <T as frame_system::Config>::Call:
        Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
    <<T as frame_system::Config>::Call as Dispatchable>::Origin: From<Option<T::AccountId>>,
{
    fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        let input = handle.input();
        let xbi: XBIFormat = custom_decode_xbi::<T>(input.to_vec())?;
        let from_local_account: T::AccountId =
            xbi_metadata_origin_2_local_account::<T>(&xbi.metadata)?;

        match xbi.instr {
            XBIInstr::CallNative { payload: _ } => {
                // let message_call = payload.take_decoded().map_err(|_| Error::FailedToDecode)?;
                // let actual_weight = match message_call.dispatch(dispatch_origin) {
                // 	Ok(post_info) => post_info.actual_weight,
                // 	Err(error_and_info) => {
                // 		// Not much to do with the result as it is. It's up to the parachain to ensure that the
                // 		// message makes sense.
                // 		error_and_info.post_info.actual_weight
                // 	},
                // }
                return Err(PrecompileFailure::Error {
                    exit_status: ExitError::Other("dispatch execution failed".into()),
                });
            }
            XBIInstr::CallEvm {
                source,
                target,
                value,
                input,
                gas_limit,
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
                access_list,
            } => {
                // Call: Dispatchable
                let call_evm_here = pallet_evm::Call::<T>::call {
                    source,
                    target,
                    value,
                    input,
                    gas_limit,
                    max_fee_per_gas,
                    max_priority_fee_per_gas,
                    nonce,
                    access_list,
                };
                call_evm_here
                    .dispatch_bypass_filter(RawOrigin::Signed(from_local_account).into())
                    .map_err(|_| PrecompileFailure::Error {
                        exit_status: ExitError::Other("dispatch execution failed".into()),
                    })?;
            }
            XBIInstr::CallWasm {
                dest: _,
                value: _,
                gas_limit: _,
                storage_deposit_limit: _,
                data: _,
            } => {
                todo!("Need wasm impl")
            }
            XBIInstr::Transfer { dest, value } => {
                let call_transfer_here = pallet_balances::Call::<T>::transfer {
                    dest: sabi::associate(dest).map_err(|_e| PrecompileFailure::Error {
                        exit_status: ExitError::Other(
                            "Failed to associate dest into T::AccountId".into(),
                        ),
                    })?,
                    value: value.try_into().map_err(|_e| PrecompileFailure::Error {
                        exit_status: ExitError::Other("Faailed to read balance from value".into()),
                    })?,
                };
                call_transfer_here
                    .dispatch_bypass_filter(RawOrigin::Signed(from_local_account).into())
                    .map_err(|_| PrecompileFailure::Error {
                        exit_status: ExitError::Other("dispatch execution failed".into()),
                    })?;
            }
            _ => todo!("Need wasm impl"),
        }

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Stopped,
            output: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
