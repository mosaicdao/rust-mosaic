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
extern crate rpassword;
extern crate web3;

use blockchain::types::bytes::Bytes;
use blockchain::{Blockchain, Kind};
pub use config::Config;
use std::error::Error;

mod blockchain;
pub mod config;

/// Runs a mosaic node with the given configuration.
/// Prints all accounts of the origin blockchain to std out.
///
/// # Arguments
///
/// * `config` - A configuration to run the mosaic node.
pub fn run(config: &Config) -> Result<(), Box<Error>> {
    let origin = match Blockchain::new(
        &Kind::Eth,
        config.origin_endpoint(),
        config.origin_validator_address(),
    ) {
        Ok(origin) => origin,
        Err(error) => {
            error!("Cannot connect to origin: {}", error);
            return Err(Box::new(error));
        }
    };
    let auxiliary = match Blockchain::new(
        &Kind::Eth,
        config.auxiliary_endpoint(),
        config.auxiliary_validator_address(),
    ) {
        Ok(auxiliary) => auxiliary,
        Err(error) => {
            error!("Cannot connect to auxiliary: {}", error);
            return Err(Box::new(error));
        }
    };

    // Example code (get accounts and sign data):
    let origin_accounts = origin.get_accounts();
    let auxiliary_accounts = auxiliary.get_accounts();

    println!("Origin accounts:");
    for account in origin_accounts {
        println!("0x{:x}", account)
    }

    println!("Auxiliary accounts:");
    for account in auxiliary_accounts {
        println!("0x{:x}", account);
    }

    let data_to_sign = Bytes::from_string("0274834951").unwrap();
    match auxiliary.sign(&data_to_sign) {
        Ok(signature) => println!("Signature: {:x}", signature),
        Err(error) => println!("Could not get signature: {}", error),
    }

    Ok(())
}
