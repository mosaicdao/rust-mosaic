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

use futures::Future;
use rlp;
use web3::contract::Options;
use web3::types::{Address, H160};

use super::ethereum::types::{Block, Error, ErrorKind};
use super::ethereum::Ethereum;

/// This is approximate gas consumed by report block operation.
const REPORT_BLOCK_ESTIMATED_GAS: i32 = 3_000_000;

/// Reports block on block store if not already reported.
///
/// # Arguments
///
/// * `block_chain` - A blockchain object.
/// * `event_loop` - The reactor's event loop to handle the tasks spawned.
/// * `block_store_address` - The address of block store.
/// * `validator_address` - The address of validator.
pub fn report_block(
    block_chain: &Ethereum,
    event_loop: &tokio_core::reactor::Handle,
    block_store_address: Address,
    validator_address: Address,
    block: &Block,
) {
    info!("Reporting block for number {:?} ", block.number);

    let encoded_block = rlp::encode(block);
    let block_hash = block.hash();

    match block_chain.contract_instance(
        block_store_address,
        include_bytes!("../contract/abi/BlockStore.json"),
    ) {
        Ok(contract) => {
            let event_loop_clone = event_loop.clone();

            let call_future = contract
                .query(
                    "isBlockReported",
                    block_hash,
                    H160::from(validator_address),
                    Options::default(),
                    None,
                ).then(move |result: Result<bool, web3::contract::Error>| {
                    let block_reported = match result {
                        Ok(is_reported) => {
                            if !is_reported {
                                let report_future = contract
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
                                    });
                                event_loop_clone.spawn(report_future);
                            }
                            Ok(!is_reported)
                        }
                        Err(e) => Err(Error::new(
                            ErrorKind::NodeError,
                            format!("Can't check if block is already reported: {}", e),
                        )),
                    };
                    info!("Block reported {:?}", block_reported);
                    Ok(())
                });

            event_loop.spawn(call_future);
        }
        Err(error) => error!("Contract instantiation failed {:?} ", error),
    }
}
