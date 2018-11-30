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
//! * Create an event factory in a separate module that implements the trait `EventFactory`.
//! * Register the new factory with the event handler in `origin_events` and/or `auxiliary_events`.
//!
//! Once a new event is added here, mosaic automatically picks up on it and executes its associated
//! action if it encounters it during execution. You don't need to add code anywhere else to use the
//! new event.

use self::block_finalized::BlockFinalizedFactory;
use self::block_reported::BlockReportedFactory;
use super::Config;
use error::{Error, ErrorKind};
use ethabi::Token;
use std::collections::HashMap;
use web3::types::{Address, Log, H256};

mod block_finalized;
mod block_reported;

/// An enum that covers all events known to mosaic.
#[derive(Debug)]
pub enum Event {
    /// A block has been finalized according to the Casper FFG protocol.
    BlockFinalized { block_hash: H256 },
    /// A block that was observed has been reported to the relevant block store.
    BlockReported { block_hash: H256 },
}

/// An event factory has a topic and can produce events from matching logs.
trait EventFactory {
    /// The topic of the logs that match the event that this factory creates.
    fn topic(&self) -> H256;

    /// This function tries to convert a log into an event and returns the Result of the
    /// conversation. Different factories will return different kinds of events.
    fn from_log(&self, &Log) -> Result<Event, Error>;
}

/// Returns an event handler that you can use to convert `log_into_event` for logs on the origin
/// chain.
///
/// # Arguments
///
/// * `config` - The mosaic configuration.
pub fn origin_event_registry(_config: &Config) -> EventRegistry {
    EventRegistry::new()
}

/// Returns an event handler that you can use to convert `log_into_event` for logs on the auxiliary
/// chain.
///
/// # Arguments
///
/// * `config` - The mosaic configuration.
pub fn auxiliary_event_registry(config: &Config) -> EventRegistry {
    let mut handler = EventRegistry::new();

    // Registering on the origin block store.
    let origin_block_store_address = config
        .origin_block_store_address()
        .expect("An origin block store address must be configured to run as a validator.");
    let origin_block_finalized_factory: BlockFinalizedFactory = Default::default();
    handler.register_event_factory(
        origin_block_store_address,
        origin_block_finalized_factory.topic(),
        Box::new(origin_block_finalized_factory),
    );
    let origin_block_reported_factory: BlockReportedFactory = Default::default();
    handler.register_event_factory(
        origin_block_store_address,
        origin_block_reported_factory.topic(),
        Box::new(origin_block_reported_factory),
    );

    // Registering block on the auxiliary block store.
    let auxiliary_block_store_address = config
        .auxiliary_block_store_address()
        .expect("An auxiliary block store address must be configured to run as a validator.");
    let auxiliary_block_finalized_factory: BlockFinalizedFactory = Default::default();
    handler.register_event_factory(
        auxiliary_block_store_address,
        auxiliary_block_finalized_factory.topic(),
        Box::new(auxiliary_block_finalized_factory),
    );
    let auxiliary_block_reported_factory: BlockReportedFactory = Default::default();
    handler.register_event_factory(
        auxiliary_block_store_address,
        auxiliary_block_reported_factory.topic(),
        Box::new(auxiliary_block_reported_factory),
    );

    handler
}

/// An event handler stores factories and converts logs into events if it has a matching factory.
pub struct EventRegistry {
    /// Maps log's addresses to a map from log's topics to related factories.
    /// This means that in order for a factory to match a log, the address and the topic must match.
    factories: HashMap<Address, HashMap<H256, Box<EventFactory>>>,
}

impl EventRegistry {
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
                return factory.from_log(&log).map(Some);
            }
        }

        Ok(None)
    }

    /// Initializes a new event handler.
    fn new() -> Self {
        EventRegistry {
            factories: HashMap::new(),
        }
    }

    /// Registers a new event factory with this handler. If the factory matches a log it will try
    /// to convert it into an event.
    ///
    /// # Arguments
    ///
    /// * `factory` - An event factory that can build an event from a log.
    fn register_event_factory(
        &mut self,
        address: Address,
        topic: H256,
        factory: Box<EventFactory>,
    ) {
        let topic_map = self.factories.entry(address).or_insert_with(HashMap::new);

        topic_map.insert(topic, factory);
    }
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

#[cfg(test)]
mod test {
    use super::*;
    use ethabi;
    use ethabi::token::{StrictTokenizer, Token, Tokenizer};
    use web3::types::Bytes;

    #[test]
    fn it_converts_logs_for_a_matching_factory() {
        let address = "1234567890123456789012345678901234567890"
            .parse::<Address>()
            .unwrap();

        let factory: BlockReportedFactory = Default::default();
        let expected_block_hash =
            "1234567890123456789012345678901234567890123456789012345678901234";
        let tokens =
            StrictTokenizer::tokenize(&ethabi::ParamType::FixedBytes(32), expected_block_hash)
                .unwrap();
        let log = build_log(address, vec![factory.topic()], &[tokens]);

        let mut handler = EventRegistry::new();
        handler.register_event_factory(address, factory.topic(), Box::new(factory));

        let event = handler.log_into_event(&log);
        match event {
            Ok(Some(event)) => match event {
                Event::BlockReported { block_hash } => {
                    assert_eq!(block_hash, expected_block_hash.parse::<H256>().unwrap())
                }
                Event::BlockFinalized { .. } => panic!("Extracted wrong type of event."),
            },
            Ok(None) => panic!("Block reported event was not converted as expected."),
            Err(error) => panic!(
                "Unexpected error when building block reported event from log: {}",
                error
            ),
        }
    }

    #[test]
    fn it_returns_none_for_unmatched_addresses() {
        let address = "1234567890123456789012345678901234567890"
            .parse::<Address>()
            .unwrap();
        let other_address = "a23456789012345678901234567890123456789a"
            .parse::<Address>()
            .unwrap();

        let factory: BlockReportedFactory = Default::default();
        let log = build_log(other_address, vec![factory.topic()], &[]);

        let mut handler = EventRegistry::new();
        handler.register_event_factory(address, factory.topic(), Box::new(factory));

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
            .parse::<Address>()
            .unwrap();

        let factory: BlockReportedFactory = Default::default();
        let log = build_log(address, vec![], &[]);

        let mut handler = EventRegistry::new();
        handler.register_event_factory(address, factory.topic(), Box::new(factory));

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
            .parse::<Address>()
            .unwrap();

        let factory: BlockReportedFactory = Default::default();
        let wrong_topic = "abcdef6adc0c092ab654c32a0ee19b8ccddafbbc780bce0a5dd193bc30aa186e"
            .parse::<H256>()
            .unwrap();
        let log = build_log(address, vec![wrong_topic], &[]);

        let mut handler = EventRegistry::new();
        handler.register_event_factory(address, factory.topic(), Box::new(factory));

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

    pub fn build_log(address: Address, topics: Vec<H256>, tokens: &[Token]) -> Log {
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
