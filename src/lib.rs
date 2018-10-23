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
extern crate web3;

use blockchain::{Blockchain, Kind};
use config::Config;
use std::error::Error;

mod blockchain;
pub mod config;

/// Runs a mosaic node with the given configuration.
/// Prints all accounts of the origin blockchain to std out.
pub fn run(config: &Config) -> Result<(), Box<Error>> {
    let origin = Blockchain::new(&Kind::Eth, config.origin_endpoint());
    let auxiliary = Blockchain::new(&Kind::Eth, config.auxiliary_endpoint());

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

    Ok(())
}
