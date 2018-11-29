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
//! To add new reactor, implement react trait and register it with block chain.

use ethereum::contract::{ContractFactory, ContractType};
use ethereum::types::block::Block;
use ethereum::types::error::Error;
use ethereum::Ethereum;
use reactor::block_reporter::BlockReporter;
use std::sync::Arc;
use Config;

mod block_reporter;

/// Anything that wants to react on block generation should implement this.
pub trait React {
    /// Defines how different reactor will react on block observation.
    /// # Arguments
    ///
    /// * `block` - The observed block.
    fn react(&self, block: &Block);
}

/// Instantiate reactors which will react on origin block generation.
///
/// # Arguments
///
/// * `origin` - A blockchain object that points to origin.
/// * `auxiliary` - A blockchain object that points to auxiliary.
/// * `contract_factory` - Contract instances factory.
/// * `config` - A configuration to register reactors.
/// * `event_loop` - A configuration to register reactors.
pub fn origin_reactors(
    _origin: Arc<Ethereum>,
    auxiliary: Arc<Ethereum>,
    contract_factory: &ContractFactory,
    config: &Config,
    event_loop: Box<tokio_core::reactor::Handle>,
) -> Result<Vec<Box<React>>, Error> {
    let mut origin_reactors: Vec<Box<React>> = Vec::new();

    contract_factory
        .get(&ContractType::OriginBlockStore)
        .map(move |contract| {
            let block_reporter = BlockReporter::new(
                contract,
                config.auxiliary_validator_address(),
                event_loop,
                auxiliary,
            );
            origin_reactors.push(Box::new(block_reporter));
            Ok(origin_reactors)
        })?
}

/// Instantiate reactors which will react on auxiliary block generation.
///
/// # Arguments
///
/// * `origin` - A blockchain object that points to origin.
/// * `auxiliary` - A blockchain object that points to auxiliary.
/// * `contract_factory` - Contract instances factory.
/// * `config` - A configuration to register reactors.
/// * `event_loop` - A configuration to register reactors.
pub fn auxiliary_reactors(
    _origin: Arc<Ethereum>,
    auxiliary: Arc<Ethereum>,
    contract_factory: &ContractFactory,
    config: &Config,
    event_loop: Box<tokio_core::reactor::Handle>,
) -> Result<Vec<Box<React>>, Error> {
    let mut auxiliary_reactors: Vec<Box<React>> = Vec::new();

    contract_factory
        .get(&ContractType::AuxiliaryBlockStore)
        .map({
            move |contract| {
                let block_reporter = BlockReporter::new(
                    contract,
                    config.auxiliary_validator_address(),
                    event_loop,
                    auxiliary,
                );
                auxiliary_reactors.push(Box::new(block_reporter));
                Ok(auxiliary_reactors)
            }
        })?
}
