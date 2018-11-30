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

pub use self::types::{Block, Signature};
use error::{Error, ErrorKind};
use event::EventRegistry;
use futures::prelude::*;
use rpassword;
use std::sync::Arc;
use std::time::Duration;
use web3::transports::Http;
use web3::types::Block as Web3Block;
use web3::types::{Address, BlockId, BlockNumber, Bytes, FilterBuilder, H160};
use web3::Web3;

mod types;

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
}

trait IntoBlock {
    /// Tries to convert itself into a `Block`. Returns a result, depending on whether the
    /// conversion was successful.
    fn into_block(&self) -> Result<Block, Error>;
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
        }
    }

    /// Stream blocks returns a `futures::stream::Stream` of `Block`s.
    ///
    /// Converts a stream of web3 blocks to a stream of blocks.
    /// The blocks contain events that were parsed from the logs based on the registered events in
    /// the event handler.
    ///
    /// It is the caller's responsibility to poll the stream, e.g. call `for_each` and put the
    /// future into a reactor.
    ///
    /// # Arguments
    ///
    /// * `event_registry` - A handler that converts raw logs from the web3 blocks into events.
    pub fn stream_blocks(
        &self,
        event_registry: Arc<EventRegistry>,
    ) -> impl Stream<Item = Block, Error = Error> {
        // Blocks filter is a future that returns a filter.
        let blocks_filter = self.web3.eth_filter().create_blocks_filter();

        // Block hashes is a stream of block hashes.
        let polling_interval = self.polling_interval;
        let block_hashes = blocks_filter
            .map(move |filter| filter.stream(polling_interval))
            .flatten_stream();

        // Web3 blocks is a stream of block futures, mapped from a stream of block hashes.
        let web3 = self.web3.clone();
        let web3_blocks = block_hashes
            .map_err(|error| {
                Error::new(
                    ErrorKind::NodeError,
                    format!("Error while streaming blocks from node: {}", error),
                )
            }).and_then(move |block_hash| {
                web3.eth()
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
        let web3 = self.web3.clone();
        blocks.and_then(move |mut block| {
            // The block number expects a `u64` as argument. `U128` cannot be safely cast to a
            // `u64`, because it is twice as long. `u64`'s max value `18446744073709551615` is
            // probably sufficient for the block number. So that would be the `U128`'s *lower* of
            // the two 64 bit unsigned integers (`low_u64`).
            let block_number: u64 = block.number.low_u64();
            let block_number = BlockNumber::from(block_number);

            // Filter for all logs of the current block.
            let filter_builder = FilterBuilder::default();
            let log_filter = filter_builder
                .from_block(block_number)
                .to_block(block_number)
                .build();

            let event_registry = Arc::clone(&event_registry);
            web3.eth()
                .logs(log_filter)
                .map_err(|error| {
                    Error::new(
                        ErrorKind::NodeError,
                        format!("Error while retrieving logs from node: {}", error),
                    )
                }).map(move |logs| {
                    for log in logs {
                        match event_registry.log_into_event(&log) {
                            // We are not interested in the case where there is no error and
                            // Ok(None) returned. It simply means that the log did not match any
                            // registered event in the event handler.
                            // So we only act if we got some event in the Ok case.
                            Ok(event) => if let Some(event) = event {
                                block.events.push(event);
                            },
                            Err(error) => {
                                warn!("Was not able to convert a log into an event: {}", error)
                            }
                        }
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
            state_root: self.state_root,
            transactions_root: self.transactions_root,
            number: match self.number {
                Some(number) => number,
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidBlock,
                        "Block has no number".to_string(),
                    ))
                }
            },
            gas_used: self.gas_used,
            gas_limit: self.gas_limit,
            timestamp: self.timestamp,
            events: vec![],
        })
    }
}
