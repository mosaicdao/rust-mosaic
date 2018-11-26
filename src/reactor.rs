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

//! This module is about different kinds of block and event reactors.

use auxiliary;
use ethereum::types::block::Block;
use ethereum::Ethereum;
use std::sync::Arc;
use web3::contract::Contract;
use web3::transports::Http;
use web3::types::Address;

use Config;

/// This enum represents all reactors which will react to block generation.
pub enum Reactor {
    BlockReporter {
        block_store: Arc<Contract<Http>>,
        validator_address: Address,
    },
}

/// Anything that wants to react on block generation should implement this.
pub trait React {
    /// Defines how different reactor will react on block observation.
    ///cargo
    /// # Arguments
    ///
    /// * `block` - The observed block.
    /// * `event_loop` - The reactor's event loop to handle the tasks spawned.
    fn react(&self, block: &Block, event_loop: &tokio_core::reactor::Handle);
}

impl React for Reactor {
    /// Defines how different reactor will react on block observation.
    ///
    /// # Arguments
    ///
    /// * `block` - The observed block.
    /// * `event_loop` - The reactor's event loop to handle the tasks spawned.
    fn react(&self, block: &Block, event_loop: &tokio_core::reactor::Handle) {
        match self {
            Reactor::BlockReporter {
                block_store,
                validator_address,
            } => {
                auxiliary::report_block(event_loop, block_store, validator_address.clone(), block);
            }
        }
    }
}

impl Reactor {
    /// Register different kind of reactors to origin and auxiliary block chain.
    ///
    /// # Arguments
    ///
    /// * `origin` - A blockchain object that points to origin.
    /// * `auxiliary` - A blockchain object that points to auxiliary.
    /// * `config` - A configuration to register reactors.
    pub fn register(origin: &mut Ethereum, auxiliary: &mut Ethereum, config: &Config) {
        match origin.contract_instance(
            config.origin_block_store_address(),
            include_bytes!("./contract/abi/BlockStore.json"),
        ) {
            Ok(contract) => {
                let origin_block_reporter = Reactor::BlockReporter {
                    block_store: Arc::new(contract),
                    validator_address: config.origin_validator_address(),
                };
                origin.register_reactor(origin_block_reporter);
            }
            Err(error) => {
                error!("Contract instantiation failed {:?} ", error);
            }
        };

        match auxiliary.contract_instance(
            config.auxiliary_block_store_address(),
            include_bytes!("./contract/abi/BlockStore.json"),
        ) {
            Ok(contract) => {
                let auxiliary_block_reporter = Reactor::BlockReporter {
                    block_store: Arc::new(contract),
                    validator_address: config.auxiliary_validator_address(),
                };
                auxiliary.register_reactor(auxiliary_block_reporter);
            }
            Err(error) => {
                error!("Contract instantiation failed {:?} ", error);
            }
        }
    }
}
