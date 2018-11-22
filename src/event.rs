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

//! Events are extracted from ethereum logs. An event results in an action on the origin chain, the
//! auxiliary chain, or both.
//!
//! To add a new event:
//! * Add it to the Event enum.
//! * Add a method to the `EventFactory` that returns a factory which creates the new type of event.
//! * Register the new factory with the event handler in `origin_events` and/or `auxiliary_events`.
//!
//! Once a new event is added here, mosaic automatically picks up on it and executes its associated
//! action if it encounters it during execution. You don't need to add code anywhere else to use the
//! new event.

use super::Config;
use error::{Error, ErrorKind};
use ethabi::{self, ParamType, Token};
use std::collections::HashMap;
use web3::types::{Log, H160, H256};

/// Returns an event handler that you can use to convert `log_into_event` for logs on the origin
/// chain.
///
/// # Arguments
///
/// * `config` - The mosaic configuration.
pub fn origin_event_handler(_config: &Config) -> EventHandler {
    EventHandler::new()
}

/// Returns an event handler that you can use to convert `log_into_event` for logs on the auxiliary
/// chain.
///
/// # Arguments
///
/// * `config` - The mosaic configuration.
pub fn auxiliary_event_handler(config: &Config) -> EventHandler {
    let mut handler = EventHandler::new();

    // Registering block reported and finalized on the origin block store.
    let origin_block_store_address = config
        .origin_block_store_address()
        .expect("An origin block store address must be configured to run as a validator.");
    handler.register_event_factory(EventFactory::new_block_finalized(
        origin_block_store_address,
    ));
    handler.register_event_factory(EventFactory::new_block_reported(origin_block_store_address));

    // Registering block reported and finalized on the auxiliary block store.
    let auxiliary_block_store_address = config
        .origin_block_store_address()
        .expect("An auxiliary block store address must be configured to run as a validator.");
    handler.register_event_factory(EventFactory::new_block_finalized(
        auxiliary_block_store_address,
    ));
    handler.register_event_factory(EventFactory::new_block_reported(
        auxiliary_block_store_address,
    ));

    handler
}

/// An enum that covers all events known to mosaic.
#[derive(Debug)]
pub enum Event {
    /// A block has been finalized according to the Casper FFG protocol.
    BlockFinalized { block_hash: H256 },
    /// A block that was observed has been reported to the relevant block store.
    BlockReported { block_hash: H256 },
}

/// An event handler stores factories and converts logs into events if it has a matching factory.
pub struct EventHandler {
    /// Maps log's addresses to a map from log's topics to related factories.
    /// This means that in order for a factory to match a log, the address and the topic must match.
    factories: HashMap<H160, HashMap<H256, EventFactory>>,
}

impl EventHandler {
    /// Convert a log into an event.
    /// Returns an `Ok(Some(Event))` if a matching factory was registered and converted the log
    /// successfully. Returns an `Ok(None)` if no matching factory was registered. Returns an `Err`
    /// if a matching factory was registered, but it could not build the event from the log.
    pub fn log_into_event(&self, log: &Log) -> Result<Option<Event>, Error> {
        if let Some(map) = self.factories.get(&log.address) {
            if log.topics.is_empty() {
                return Ok(None);
            }

            // The canonical identifier for a log is always the first topic.
            if let Some(factory) = map.get(&log.topics[0]) {
                // Try to build the event with the matching factory.
                match (factory.from_log)(&log) {
                    Ok(event) => return Ok(Some(event)),
                    Err(error) => return Err(error),
                }
            }
        }

        Ok(None)
    }

    /// Initializes a new event handler.
    fn new() -> Self {
        EventHandler {
            factories: HashMap::new(),
        }
    }

    /// Registers a new event factory with this handler. If the factory matches a log it will try
    /// to convert it into an event.
    ///
    /// # Arguments
    ///
    /// * `factory` - An event factory that can build an event from a log.
    fn register_event_factory(&mut self, factory: EventFactory) {
        let topic_map = self
            .factories
            .entry(factory.address)
            .or_insert_with(HashMap::new);

        topic_map.insert(factory.topic, factory);
    }
}

/// An event factory produces events from logs that match the set address and topic.
/// The topic must equal the first topic in the log's list of topics. It is the SHA3 hash of the
/// canonical event signature.
struct EventFactory {
    /// The log's address must equal this address.
    address: H160,
    /// The log's first topic must equal this topic.
    topic: H256,
    /// This function tries to convert the log into an event and returns a Result.
    from_log: Box<Fn(&Log) -> Result<Event, Error>>,
}

impl EventFactory {
    /// Returns a new factory that builds "Block Finalized" events.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the contract that emits this event.
    fn new_block_finalized(address: H160) -> Self {
        Self {
            address,
            // `sha3("BlockFinalised(bytes32)")`
            topic: "2b6cea6adc0c092ab654c32a0ee19b8ccddafbbc780bce0a5dd193bc30aa186e"
                .parse::<H256>()
                .unwrap(),
            from_log: Box::new(Self::block_finalized_from_log),
        }
    }

    /// Returns a new factory that builds "Block Reported" events.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the contract that emits this event.
    fn new_block_reported(address: H160) -> Self {
        Self {
            address,
            // `sha3("BlockReported(bytes32)")`
            topic: "721303f9f13058e7a8abd8036b2897d3cee27492b247eceddd6203ff601c006b"
                .parse::<H256>()
                .unwrap(),
            from_log: Box::new(Self::block_reported_from_log),
        }
    }

    /// Tries to build a "Block Finalized" event from a log entry.
    ///
    /// # Arguments
    ///
    /// * `log` - The log that shall be converted into an event.
    fn block_finalized_from_log(log: &Log) -> Result<Event, Error> {
        let log_data = ethabi::decode(&[ParamType::FixedBytes(32)], &log.data.0[..]);
        let block_hash: H256 = match log_data {
            // There should only be a single bytes32 in the vector of decoded elements.
            Ok(decoded_elements) => match Self::block_hash_from_abi(&decoded_elements[0]) {
                Ok(block_hash) => block_hash,
                Err(error) => return Err(error),
            },
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::AbiError,
                    format!(
                        "Error when doing ABI decoding of 'block finalized' event: {}",
                        error
                    ),
                ))
            }
        };

        Ok(Event::BlockFinalized { block_hash })
    }

    /// Tries to build a "Block Reported" event from a log entry.
    ///
    /// # Arguments
    ///
    /// * `log` - The log that shall be converted into an event.
    fn block_reported_from_log(log: &Log) -> Result<Event, Error> {
        let log_data = ethabi::decode(&[ParamType::FixedBytes(32)], &log.data.0[..]);
        let block_hash: H256 = match log_data {
            // There should only be a single bytes32 in the vector of decoded elements.
            Ok(decoded_elements) => match Self::block_hash_from_abi(&decoded_elements[0]) {
                Ok(block_hash) => block_hash,
                Err(error) => return Err(error),
            },
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::AbiError,
                    format!(
                        "Error when doing ABI decoding of 'block reported' event: {}",
                        error
                    ),
                ))
            }
        };

        Ok(Event::BlockReported { block_hash })
    }

    /// Converts a Token that resulted from ABI parsing into a block hash.
    ///
    /// # Arguments
    ///
    /// * `token` - The token that resulted from the ABI parsing.
    fn block_hash_from_abi(token: &Token) -> Result<H256, Error> {
        match token {
            Token::FixedBytes(bytes) => Ok(bytes[..].into()),
            _ => Err(Error::new(
                ErrorKind::AbiError,
                "Unexpected return type after ABI decoding 'block reported' event".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ethabi;
    use ethabi::token::{StrictTokenizer, Token, Tokenizer};
    use web3::types::Bytes;

    #[test]
    fn it_converts_logs_for_block_reported() {
        let address = "1234567890123456789012345678901234567890"
            .parse::<H160>()
            .unwrap();
        let factory = EventFactory::new_block_reported(address);

        let mut handler = EventHandler::new();
        handler.register_event_factory(factory);

        let topic = "721303f9f13058e7a8abd8036b2897d3cee27492b247eceddd6203ff601c006b";

        let expected_block_hash =
            "1234567890123456789012345678901234567890123456789012345678901234";
        let tokens =
            StrictTokenizer::tokenize(&ethabi::ParamType::FixedBytes(32), expected_block_hash)
                .unwrap();

        let log = build_log(address, vec![topic.parse::<H256>().unwrap()], &[tokens]);
        let event = handler.log_into_event(&log);
        match event {
            Ok(Some(event)) => match event {
                Event::BlockReported { block_hash } => {
                    assert_eq!(block_hash, expected_block_hash.parse::<H256>().unwrap())
                }
                Event::BlockFinalized { block_hash: _ } => panic!("Extracted wrong type of event."),
            },
            Ok(None) => panic!("Block reported event was not converted as expected."),
            Err(error) => panic!(
                "Unexpected error when building block reported event from log: {}",
                error
            ),
        }
    }

    #[test]
    fn it_converts_logs_for_block_finalized() {
        let address = "1234567890123456789012345678901234567890"
            .parse::<H160>()
            .unwrap();
        let factory = EventFactory::new_block_finalized(address);

        let mut handler = EventHandler::new();
        handler.register_event_factory(factory);

        let topic = "2b6cea6adc0c092ab654c32a0ee19b8ccddafbbc780bce0a5dd193bc30aa186e";

        let expected_block_hash =
            "a234567890123456789012345678901234567890123456789012345678901234";
        let tokens =
            StrictTokenizer::tokenize(&ethabi::ParamType::FixedBytes(32), expected_block_hash)
                .unwrap();

        let log = build_log(address, vec![topic.parse::<H256>().unwrap()], &[tokens]);
        let event = handler.log_into_event(&log);
        match event {
            Ok(Some(event)) => match event {
                Event::BlockReported { block_hash: _ } => panic!("Extracted wrong type of event."),
                Event::BlockFinalized { block_hash } => {
                    assert_eq!(block_hash, expected_block_hash.parse::<H256>().unwrap())
                }
            },
            Ok(None) => panic!("Block finalized event was not converted as expected."),
            Err(error) => panic!(
                "Unexpected error when building block finalized event from log: {}",
                error
            ),
        }
    }

    #[test]
    fn it_returns_none_for_unmatched_addresses() {
        let address = "1234567890123456789012345678901234567890"
            .parse::<H160>()
            .unwrap();
        let other_address = "a23456789012345678901234567890123456789a"
            .parse::<H160>()
            .unwrap();
        let factory = EventFactory::new_block_finalized(address);

        let mut handler = EventHandler::new();
        handler.register_event_factory(factory);

        let topic = "2b6cea6adc0c092ab654c32a0ee19b8ccddafbbc780bce0a5dd193bc30aa186e";

        let log = build_log(other_address, vec![topic.parse::<H256>().unwrap()], &[]);
        let event = handler.log_into_event(&log);
        match event {
            Ok(Some(_)) => panic!("Should not have matched any event."),
            Ok(None) => (),
            Err(error) => panic!(
                "Unexpected error when not matching any event from log: {}",
                error
            ),
        }
    }

    #[test]
    fn it_returns_none_for_logs_with_empty_topics() {
        let address = "1234567890123456789012345678901234567890"
            .parse::<H160>()
            .unwrap();
        let factory = EventFactory::new_block_finalized(address);

        let mut handler = EventHandler::new();
        handler.register_event_factory(factory);

        let log = build_log(address, vec![], &[]);
        let event = handler.log_into_event(&log);
        match event {
            Ok(Some(_)) => panic!("Should not have matched any event."),
            Ok(None) => (),
            Err(error) => panic!(
                "Unexpected error when not matching any event from log: {}",
                error
            ),
        }
    }

    #[test]
    fn it_returns_none_for_unmatched_topic() {
        let address = "1234567890123456789012345678901234567890"
            .parse::<H160>()
            .unwrap();
        let factory = EventFactory::new_block_finalized(address);

        let mut handler = EventHandler::new();
        handler.register_event_factory(factory);

        let topic = "abcdef6adc0c092ab654c32a0ee19b8ccddafbbc780bce0a5dd193bc30aa186e";

        let log = build_log(address, vec![topic.parse::<H256>().unwrap()], &[]);
        let event = handler.log_into_event(&log);
        match event {
            Ok(Some(_)) => panic!("Should not have matched any event."),
            Ok(None) => (),
            Err(error) => panic!(
                "Unexpected error when not matching any event from log: {}",
                error
            ),
        }
    }

    fn build_log(address: H160, topics: Vec<H256>, tokens: &[Token]) -> Log {
        let encoded = ethabi::encode(tokens);
        let data: Bytes = encoded.into();

        Log {
            address,
            topics,
            data,
            block_hash: None,
            block_number: None,
            transaction_hash: None,
            transaction_index: None,
            log_index: None,
            transaction_log_index: None,
            log_type: None,
            removed: None,
        }
    }
}
