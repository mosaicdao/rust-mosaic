// Copyright 2018 OpenST Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//!This module is about interaction with auxiliary block chain.

use futures::Future;
use web3::contract::{Contract, Options};
use web3::transports::Http;
use web3::types::{H160, Address};
use rlp;

use super::ethereum::Ethereum;
use super::ethereum::types::Block;

const REPORT_BLOCK_ESTIMATED_GAS:i32 = 3_000_000;

/// # Arguments
///
/// * `blockchain` - A blockchain object.
/// * `event_loop` - The reactor's event loop to handle the tasks spawned.
/// * `block_store_address` - The address of origin block store.
/// * `validator_address` - The address of origin validator address.
pub fn report_block(
    block_chain: &Ethereum,
    event_loop: &tokio_core::reactor::Handle,
    block_store_address: Address,
    validator_address: Address,
    block: &Block
) {
    let encoded_block = rlp::encode(block);

    let contract: Contract<Http> = block_chain.contract_instance(
        block_store_address,
        include_bytes!( "../contract/abi/BlockStore.json"),
    );

    let call_future = contract
        .call("reportBlock", encoded_block, H160::from(validator_address), Options::with(|opt| {
            opt.gas = Some(REPORT_BLOCK_ESTIMATED_GAS.into())
        }))
        .then( |tx| {
            println!("Block reported got tx: {:?}", tx);
            Ok(())
        });

    event_loop.spawn(call_future);
}

