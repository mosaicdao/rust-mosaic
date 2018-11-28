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

use ethereum::contract::{ContractInstances, ContractType};
use ethereum::types::block::Block;
use ethereum::types::error::Error;
use ethereum::Ethereum;
use reactor::block_reporter::BlockReporter;
use Config;

mod block_reporter;

/// Anything that wants to react on block generation should implement this.
pub trait React {
    /// Defines how different reactor will react on block observation.
    /// # Arguments
    ///
    /// * `block` - The observed block.
    /// * `event_loop` - The reactor's event loop to handle the tasks spawned.
    fn react(&self, block: &Block, event_loop: &tokio_core::reactor::Handle);
}

/// Register different kind of reactors to origin and auxiliary block chain.
///
/// # Arguments
///
/// * `origin` - A blockchain object that points to origin.
/// * `auxiliary` - A blockchain object that points to auxiliary.
/// * `config` - A configuration to register reactors.
pub fn register(
    origin: &mut Ethereum,
    auxiliary: &mut Ethereum,
    contract_instances: &ContractInstances,
    config: &Config,
) -> Result<(), Error> {
    contract_instances
        .get(&ContractType::AuxiliaryBlockStore)
        .map(|contract| {
            auxiliary.register_reactor(Box::new(BlockReporter::new(
                contract,
                config.auxiliary_validator_address(),
            )))
        })?;

    contract_instances
        .get(&ContractType::OriginBlockStore)
        .map(|contract| {
            origin.register_reactor(Box::new(BlockReporter::new(
                contract,
                config.auxiliary_validator_address(),
            )))
        })?;
    Ok(())
}

