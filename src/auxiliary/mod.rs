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

//! This module is about interaction with auxiliary block chain.

use futures::future::Either;
use futures::future::IntoFuture;
use futures::Future;
use rlp;
use std::sync::Arc;
use web3::contract::Contract;
use web3::contract::Options;
use web3::transports::Http;
use web3::types::Address;

use super::ethereum::types::Block;

/// This is gas consumed by report block operation. This is calculated by estimate gas
/// function from web3.
const REPORT_BLOCK_ESTIMATED_GAS: i32 = 3_000_000;

/// Reports block on block store if not already reported.
///
/// # Arguments
///
/// * `event_loop` - The reactor's event loop to handle the tasks spawned.
/// * `block_store_contract` - The block store contract instance.
/// * `validator_address` - The address of validator.
/// * `block` - The observed block which needs to be reported.
pub fn report_block(
    event_loop: &tokio_core::reactor::Handle,
    block_store_contract: &Arc<Contract<Http>>,
    validator_address: Address,
    block: &Block,
) {
    info!("Reporting block for number {:?} ", block.number);

    let encoded_block = rlp::encode(block);
    let block_hash = block.hash();
    let block_store_contract = Arc::clone(&block_store_contract);
    let call_future = block_store_contract
        .query(
            "isBlockReported",
            block_hash,
            validator_address,
            Options::default(),
            None,
        ).then(
            move |result: Result<bool, web3::contract::Error>| match result {
                Ok(is_reported) => if is_reported {
                    Either::A(Ok(()).into_future())
                } else {
                    Either::B(
                        block_store_contract
                            .call(
                                "reportBlock",
                                encoded_block,
                                validator_address.into(),
                                Options::with(|opt| {
                                    opt.gas = Some(REPORT_BLOCK_ESTIMATED_GAS.into())
                                }),
                            ).then(move |tx| {
                                info!("Block reported got tx: {:?}", tx);
                                Ok(())
                            }),
                    )
                },
                Err(error) => {
                    error!(
                        "Error while checking if block is already reported{:?}",
                        error
                    );
                    // Event loop spawn expects certain types. It doesn't support err types.
                    Either::A(Ok(()).into_future())
                }
            },
        );
    event_loop.spawn(call_future);
}
