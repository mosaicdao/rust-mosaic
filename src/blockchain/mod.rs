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

//! This module provides an API to interact with blockchains, e.g. Ethereum.

pub use self::types::*;
use futures::prelude::*;
use std::time::Duration;

mod ethereum;
pub mod types;

/// Kind only represents what kind a blockchain is without any implementation.
pub enum BlockchainKind {
    Eth,
}

/// A blockchain is a connection to a blockchain.
pub enum Blockchain {
    Eth(ethereum::Ethereum),
}

impl Blockchain {
    /// Creates a new blockchain of the given kind pointing to the given address.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind that the blockchain shall be.
    /// * `endpoint` - The endpoint of a node of the blockchain.
    /// * `validator` - The address of the validator to sign messages.
    /// * `polling_interval` - The duration in between two calls to the node to poll for new blocks.
    /// * `event_loop` - The event loop handle.
    pub fn new(
        kind: &BlockchainKind,
        endpoint: &str,
        validator: Address,
        polling_interval: Duration,
        event_loop: Box<tokio_core::reactor::Handle>,
    ) -> Self {
        match kind {
            BlockchainKind::Eth => Blockchain::Eth(ethereum::Ethereum::new(
                endpoint,
                validator,
                polling_interval,
                event_loop,
            )),
        }
    }

    /// Stream blocks returns a `futures::stream::Stream` of `Block`s.
    ///
    /// It is the caller's responsibility to poll the stream, e.g. call `for_each` and put the
    /// future into a reactor.
    pub fn stream_blocks(&self) -> impl Stream<Item = Block, Error = Error> {
        match self {
            Blockchain::Eth(ethereum) => ethereum.stream_blocks(),
        }
    }

    /// Returns all accounts on this blockchain.
    pub fn get_accounts(&self) -> impl Future<Item = Vec<Address>, Error = Error> {
        match self {
            Blockchain::Eth(ethereum) => ethereum.get_accounts(),
        }
    }

    /// Signs the given data.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to sign.
    ///
    /// # Returns
    ///
    /// Returns a `Signature` of the signed data.
    pub fn sign(&self, data: Bytes) -> impl Future<Item = Signature, Error = Error> {
        match self {
            Blockchain::Eth(ethereum) => ethereum.sign(data),
        }
    }
}
