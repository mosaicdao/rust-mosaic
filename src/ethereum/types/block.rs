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

//! This module covers blocks.

use std::fmt::{self, Display, Formatter};
use web3::types::{Address, Bytes, H256, U128, U256};

/// A block represents a block of a blockchain.
#[derive(Debug)]
pub struct Block {
    /// The block hash of this block.
    pub hash: H256,
    pub parent_hash: H256,
    pub state_root: H256,
    pub transactions_root: H256,
    pub number: U128,
    pub gas_used: U256,
    pub gas_limit: U256,
    pub timestamp: U256,
    pub events: Vec<Event>,
}

impl Display for Block {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Block ({:x})", self.hash);

        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub address: Address,
    pub topics: Vec<H256>,
    pub data: Bytes,
    pub block_hash: Option<H256>,
    pub block_number: Option<U256>,
    pub transaction_hash: Option<H256>,
    pub transaction_index: Option<U256>,
    pub log_index: Option<U256>,
    pub transaction_log_index: Option<U256>,
    pub log_type: Option<String>,
    pub removed: Option<bool>,
}
