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

///! This module manages contract instances.
///! To add new contract instance add a new enum type and also initialize contract in initialize method.
use ethereum::types::error::Error;
use ethereum::types::error::ErrorKind;
use ethereum::Ethereum;
use std::collections::HashMap;
use std::sync::Arc;
use web3::contract::Contract;
use web3::transports::Http;
use Config;

/// This enum represents types of contract.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ContractType {
    OriginBlockStore,
    AuxiliaryBlockStore,
}

/// This struct stores map of contract type and  instances.
pub struct ContractFactory {
    contracts: HashMap<ContractType, Arc<Contract<Http>>>,
}

impl ContractFactory {
    ///Creates instance of contract instances struct with empty map.
    pub fn new() -> Self {
        let contracts: HashMap<ContractType, Arc<Contract<Http>>> = HashMap::new();
        ContractFactory { contracts }
    }
    /// This instantiate all the contracts and stores them into map.
    /// This throws error if initialization fails.
    ///
    /// # Arguments
    ///
    /// * `_origin` - Origin block chain instance.
    /// * `auxiliary` - Auxiliary block chain instance.
    /// * `config` - configuration of mosaic node.
    pub fn initialize(
        &mut self,
        _origin: Arc<Ethereum>,
        auxiliary: Arc<Ethereum>,
        config: &Config,
    ) -> Result<(), Error> {
        let contracts = &mut self.contracts;

        auxiliary
            .contract_instance(
                config.origin_block_store_address(),
                include_bytes!("../contract/abi/BlockStore.json"),
            ).map(|instance| contracts.insert(ContractType::OriginBlockStore, Arc::new(instance)))?;

        auxiliary
            .contract_instance(
                config.auxiliary_block_store_address(),
                include_bytes!("../contract/abi/BlockStore.json"),
            ).map(|instance| {
                contracts.insert(ContractType::AuxiliaryBlockStore, Arc::new(instance))
            })?;

        Ok(())
    }
    /// This returns contract instance.
    /// This throws error if contract instance doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `contract_type` - Type of contract.
    pub fn get(&self, contract_type: &ContractType) -> Result<Arc<Contract<Http>>, Error> {
        match self.contracts.get(contract_type) {
            Some(instance) => Ok(Arc::clone(instance)),
            None => Err(Error::new(
                ErrorKind::NodeError,
                format!(
                    "Contract instance not available for the contract {:?} ",
                    contract_type
                ),
            )),
        }
    }
}
