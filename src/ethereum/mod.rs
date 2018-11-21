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

//! This module implements the connection to an Ethereum blockchain.

use ethereum::types::{Block, Error, ErrorKind, Event, Signature};
use futures::prelude::*;
use rpassword;
use std::time::Duration;
use web3::transports::Http;
use web3::types::Block as Web3Block;
use web3::types::H256 as Web3H256;
use web3::types::U128 as Web3U128;
use web3::types::U256 as Web3U256;
use web3::contract::Contract;
use web3::types::{Address, BlockId, BlockNumber, Bytes, FilterBuilder, Log, H160};
use web3::Web3;

use super::auxiliary;

pub mod types;

/// This struct stores a connection to an Ethereum node.
#[derive(Clone)]
pub struct Ethereum {
    web3: Web3<Http>,
    validator: H160,
    /// The password to unlock the validator account on the node.
    password: String,
    /// The polling interval defines the duration in between two calls to the node to poll for new
    /// blocks.
    polling_interval: Duration,
    /// A handle to the event loop that runs mosaic.
    event_loop: Box<tokio_core::reactor::Handle>,
    /// List of block observers. These are notified when any new block is generated.
    observers: Vec<Reactor>,
}

/// This enum represents all reactors which will react to block generation.
#[derive(Debug, Clone)]
pub enum Reactor {
    BlockReporter { block_store_address: Address, validator_address: Address },
}

trait IntoBlock {
    fn into_block(&self) -> Result<Block, Error>;
}

/// Anything that wants to react on block generation should implement this.
trait Observe {
    fn observe(&self, value: &Block, block_chain: &Ethereum);
}

impl Observe for Reactor {
    fn observe(&self, block: &Block, block_chain: &Ethereum) {
        match &self {
            Reactor::BlockReporter { block_store_address, validator_address } => {
                auxiliary::report_block(
                    &block_chain,
                    &block_chain.event_loop,
                    block_store_address.clone(),
                    validator_address.clone(),
                    block,
                );
            }
        }
    }
}

impl Ethereum {
    /// Creates a new instance of Ethereum pointing to the given address.
    /// Reads the password to unlock the account in the ethereum node from `stdin`.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The address of an ethereum node.
    /// * `validator` - The address of the validator to sign and send messages from.
    /// * `polling_interval` - The duration in between two calls to the node to poll for new blocks.
    /// * `event_loop` - A handle to the event loop that runs mosaic.
    pub fn new(
        endpoint: &str,
        validator: H160,
        polling_interval: Duration,
        event_loop: Box<tokio_core::reactor::Handle>,
    ) -> Self {
        let http = Http::with_event_loop(endpoint, &event_loop, 5)
            .expect("Could not initialize ethereum HTTP connection");
        let web3 = Web3::new(http);

        let password = rpassword::prompt_password_stdout(&format!(
            "Please enter the password for account {:x}: ",
            &validator,
        )).unwrap();

        Ethereum {
            web3,
            validator,
            password,
            polling_interval,
            event_loop,
            observers: Vec::new(),
        }
    }

    /// Stream blocks returns a `futures::stream::Stream` of `Block`s.
    ///
    /// Converts a stream of web3 blocks to a stream of blocks.
    ///
    /// It is the caller's responsibility to poll the stream, e.g. call `for_each` and put the
    /// future into a reactor.
    pub fn stream_blocks(&self) -> impl Stream<Item = Block, Error = Error> {
        // Blocks filter is a future that returns a filter.
        let blocks_filter = self.web3.eth_filter().create_blocks_filter();

        // Block hashes is a stream of block hashes.
        let polling_interval = self.polling_interval;
        let block_hashes = blocks_filter
            .map(move |filter| filter.stream(polling_interval))
            .flatten_stream();

        // Web3 blocks is a stream of block futures, mapped from a stream of block hashes.
        let web3_clone = self.web3.clone();
        let web3_blocks = block_hashes
            .map_err(|error| {
                Error::new(
                    ErrorKind::NodeError,
                    format!("Error while streaming blocks from node: {}", error),
                )
            }).and_then(move |block_hash| {
                web3_clone
                    .eth()
                    .block(BlockId::from(block_hash))
                    .map_err(|error| {
                        Error::new(
                            ErrorKind::NodeError,
                            format!("Was not able to retrieve block: {}", error),
                        )
                    })
            });

        // Returns a stream of blocks, mapped from a stream of web3 block futures.
        let blocks = web3_blocks.and_then(|web3_block| match web3_block {
            // Mapping web3 block Option to a Block.
            // Wrapping in Ok() as it has to return an IntoFuture.
            Some(web3_block) => match web3_block.into_block() {
                Ok(block) => Ok(block),
                Err(error) => Err(Error::new(
                    ErrorKind::NodeError,
                    format!("Could not convert block from web3: {}", error),
                )),
            },
            None => Err(Error::new(
                ErrorKind::NodeError,
                "No block found".to_string(),
            )),
        });

        // Get all events for that block from the node and add them to the block struct.
        let web3_clone = self.web3.clone();
        blocks.and_then(move |mut block| {
            let block_number: u64 = block.number.low_u64();
            let block_number = BlockNumber::from(block_number);

            // Filter for all logs of the current block.
            let filter_builder = FilterBuilder::default();
            let log_filter = filter_builder
                .from_block(block_number)
                .to_block(block_number)
                .build();

            web3_clone
                .eth()
                .logs(log_filter)
                .map_err(|error| {
                    Error::new(
                        ErrorKind::NodeError,
                        format!("Error while retrieving logs from node: {}", error),
                    )
                }).map(|logs| {
                    for log in logs {
                        block.events.push(log.into());
                    }

                    block
                })
        })
    }

    /// Uses web3 to retrieve the accounts.
    /// Converts them to blockchain addresses and returns all addresses in a
    /// vector.
    pub fn get_accounts(&self) -> impl Future<Item = Vec<Address>, Error = Error> {
        self.web3.eth().accounts().map_err(|error| {
            Error::new(
                ErrorKind::NodeError,
                format!("Was not able to retrieve accounts: {}", error),
            )
        })
    }

    /// Uses web3 to sign the given data.
    /// Converts the signature to a blockchain signature.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to sign.
    ///
    /// # Returns
    ///
    /// Returns a `Signature` of the signed data.
    pub fn sign(&self, data: Bytes) -> impl Future<Item = Signature, Error = Error> {
        let web3_clone = self.web3.clone();
        let validator = self.validator;

        self.unlock_account(None).and_then(move |_| {
            web3_clone.eth().sign(validator, data).map_err(|error| {
                Error::new(
                    ErrorKind::NodeError,
                    format!("Was not able to sign data: {}", error),
                )
            })
        })
    }

    /// Create contract instance
    ///
    /// # Arguments
    ///
    /// * `contract_address` -  The address of contract.
    /// * `abi` - ABI of contract.
    ///
    /// # Returns
    ///
    /// Returns a `contract` instance.
    pub fn contract_instance(&self, contract_address: Address, abi: &[u8]) -> Contract<Http> {
        Contract::from_json(
            self.web3.eth(),
            H160::from(contract_address),
            abi,
        ).unwrap()
    }

    /// Unlocks the validator account of this ethereum instance using the stored password.
    ///
    /// # Arguments
    ///
    /// * `duration` - If given, will unlock for the duration in seconds. Otherwise for a single
    /// transaction.
    ///
    /// # Panics
    ///
    /// Panics if it cannot unlock the account.
    fn unlock_account(&self, duration: Option<u16>) -> impl Future<Item = bool, Error = Error> {
        self.web3
            .personal()
            .unlock_account(self.validator, &self.password, duration)
            .map_err(|error| {
                Error::new(
                    ErrorKind::NodeError,
                    format!("Was not able to unlock account: {}", error),
                )
            })
    }

    /// Register a block observer.
    ///
    /// # Arguments
    ///
    /// * `observer` - Any object which implements observer traits
    ///
    pub fn register_observer(&mut self, observer: Reactor) {
        self.observers.push(observer);
    }

    /// Notify all the block observers
    ///
    /// # Arguments
    ///
    /// * `block` - block to notify
    ///
    pub fn notify_all_observers(&mut self, block: &Block) {

        for observer in &self.observers {
            observer.observe(block, &self);
        }
    }

}

impl From<Log> for Event {
    fn from(log: Log) -> Event {
        Event {
            address: log.address,
            topics: log.topics,
            data: log.data,
            block_hash: log.block_hash,
            block_number: log.block_number,
            transaction_hash: log.transaction_hash,
            transaction_index: log.transaction_index,
            log_index: log.log_index,
            transaction_log_index: log.transaction_log_index,
            log_type: log.log_type,
            removed: log.removed,
        }
    }
}

impl<TX> IntoBlock for Web3Block<TX> {
    /// Tries to convert a web3 block into a `Block`.
    ///
    /// Fails if mandatory fields are missing.
    fn into_block(&self) -> Result<Block, Error> {
        Ok(Block {
            hash: match self.hash {
                Some(hash) => hash,
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidBlock,
                        "Block has no hash".to_string(),
                    ));
                }
            },
            parent_hash: self.parent_hash,
            uncles_hash: self.parent_hash,
            author: self.author,
            state_root: self.state_root,
            transactions_root: self.transactions_root,
            receipts_root: self.transactions_root,
            logs_bloom: self.logs_bloom,
            total_difficulty: self.total_difficulty,
            number: match self.number {
                Some(number) => number,
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidBlock,
                        "Block has no number".to_string(),
                    ));
                }
            },
            gas_limit: self.gas_limit,
            gas_used: self.gas_used,
            timestamp: self.timestamp,
            extra_data: self.extra_data.clone(),
            mix_data: self.transactions_root,
            nonce: self.difficulty,
            events: vec![],
        })
    }
}
