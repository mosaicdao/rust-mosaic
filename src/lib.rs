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

//! This crate implements a mosaic node.
//! Mosaic nodes run to:
//!  - verify or validate for one or more Mosaic cores
//!  - do facilitator work

#[macro_use]
extern crate log;
extern crate futures;
extern crate rpassword;
extern crate tokio_core;
extern crate web3;
extern crate rustc_hex;

use blockchain::{Blockchain, BlockchainKind};
pub use config::Config;
use std::error::Error;

mod blockchain;
pub mod config;
mod observer;
mod auxiliary;

/// Runs a mosaic node with the given configuration.
/// Prints all accounts of the origin blockchain to std out.
///
/// # Arguments
///
/// * `config` - A configuration to run the mosaic node.
pub fn run(config: &Config) -> Result<(), Box<Error>> {
    let mut event_loop =
        tokio_core::reactor::Core::new().expect("Could not initialize tokio event loop");
    let origin = Blockchain::new(
        &BlockchainKind::Eth,
        config.origin_endpoint(),
        config.origin_validator_address(),
        config.origin_polling_interval(),
        Box::new(event_loop.handle()),
    );
    let auxiliary = Blockchain::new(
        &BlockchainKind::Eth,
        config.auxiliary_endpoint(),
        config.auxiliary_validator_address(),
        config.auxiliary_polling_interval(),
        Box::new(event_loop.handle()),
    );

    observer::run(&origin, &auxiliary, &event_loop.handle());

    auxiliary::run(
        &origin,
        &event_loop.handle(),
        config.origin_block_store_address(),
        config.auxiliary_validator_address(),
    );

    loop {
        event_loop.turn(None);
    }
}
