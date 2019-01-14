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

use rlp;
use rlp::{Encodable, RlpStream};
use std::fmt::{self, Display, Formatter};
use tiny_keccak::Keccak;
use web3::types::{Address, Bytes, H160, H2048, H256, U128, U256};

/// A block represents a block of a blockchain.
#[derive(Debug)]
pub struct Block {
    /// The block hash of this block.
    pub hash: H256,
    pub parent_hash: H256,
    pub uncles_hash: H256,
    pub author: H160,
    pub state_root: H256,
    pub transactions_root: H256,
    pub receipts_root: H256,
    pub logs_bloom: H2048,
    pub total_difficulty: U256,
    pub number: U128,
    pub gas_limit: U256,
    pub gas_used: U256,
    pub timestamp: U256,
    pub extra_data: Bytes,
    pub mix_data: H256,
    pub nonce: U256,
    pub events: Vec<Event>,
}

impl Display for Block {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Block ({:x})", self.hash);

        Ok(())
    }
}

impl Block {
    /// Calculate hash of block.
    pub fn hash(&self) -> H256 {
        let encoded_block = rlp::encode(self);
        let mut res: [u8; 32] = [0; 32];
        Keccak::keccak256(encoded_block.as_slice(), &mut res);
        H256::from(res)
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

impl Encodable for Block {
    /// RLP encode block
    ///
    /// # Arguments
    ///
    /// * `rlp_stream` - Appendable rlp encoder.
    fn rlp_append(&self, rlp_stream: &mut RlpStream) {
        rlp_stream.begin_list(15);

        rlp_stream.append(&self.parent_hash);
        rlp_stream.append(&self.uncles_hash);
        rlp_stream.append(&self.author);
        rlp_stream.append(&self.state_root);
        rlp_stream.append(&self.transactions_root);
        rlp_stream.append(&self.receipts_root);
        rlp_stream.append(&self.logs_bloom);
        rlp_stream.append(&self.total_difficulty);
        rlp_stream.append(&self.number);
        rlp_stream.append(&self.gas_limit);
        rlp_stream.append(&self.gas_used);
        rlp_stream.append(&self.timestamp);
        rlp_stream.append(&self.hash);
        rlp_stream.append(&self.mix_data);
        rlp_stream.append(&self.nonce);
    }
}
