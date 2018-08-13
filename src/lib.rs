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
//!  - validate auxiliary systems
//!  - commit an origin chain onto an auxiliary chain
//!  - commit an auxiliary chain onto an origin chain

#[macro_use]
extern crate log;
extern crate web3;

use blockchain::{Blockchain, Kind};
use config::Config;
use std::error::Error;

mod blockchain;
pub mod config;

/// Runs a mosaic node with the given configuration.
/// Prints all accounts of the origin blockchain to std out.
pub fn run(config: Config) -> Result<(), Box<Error>> {
    let blockchain = Blockchain::new(Kind::Eth, config.origin_endpoint().to_owned());

    let accounts = blockchain.get_accounts();

    println!("Accounts:");
    for account in accounts {
        println!("0x{:x}", account)
    }

    Ok(())
}
