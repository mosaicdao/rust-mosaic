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
//!  - validate utility systems
//!  - commit a value chain onto a utility chain
//!  - commit a utility chain onto a value chain

#[macro_use]
extern crate log;
extern crate web3;

use std::env;
use std::error::Error;

mod blockchain;

// Environment variables and their defaults
const ENV_SOURCE_ADDRESS: &str = "MOSAIC_SOURCE_ADDRESS";
const ENV_TARGET_ADDRESS: &str = "MOSAIC_TARGET_ADDRESS";
const DEFAULT_SOURCE_ADDRESS: &str = "http://127.0.0.1:8545";
const DEFAULT_TARGET_ADDRESS: &str = "http://127.0.0.1:8546";

/// Global config for running a mosaic node.
pub struct Config {
    /// Address of the source chain, e.g. "127.0.0.1:8485"
    source_address: String,
    /// Address of the target chain, e.g. "127.0.0.1:8486"
    target_address: String,
}

impl Config {
    /// Reads the configuration from environment variables and creates a new Config from them. In
    /// case an environment variable is not set, a default fallback will be used.
    pub fn new() -> Result<Config, &'static str> {
        // Read source address from env and set it or fallback to default
        let source_address = env::var(ENV_SOURCE_ADDRESS);
        let source_address = match source_address {
            Ok(address) => address,
            Err(_) => {
                info!("No source chain address given, falling back to default.");
                DEFAULT_SOURCE_ADDRESS.to_string()
            }
        };

        // Read target address from env and set it or fallback to default
        let target_address = env::var(ENV_TARGET_ADDRESS);
        let target_address = match target_address {
            Ok(address) => address,
            Err(_) => {
                info!("No target chain address given, falling back to default.");
                DEFAULT_TARGET_ADDRESS.to_string()
            }
        };

        info!("Using source chain address: {}", source_address);
        info!("Using target chain address: {}", target_address);

        Ok(Config {
            source_address,
            target_address,
        })
    }
}

/// Runs a mosaic node with the given configuration.
/// Prints all accounts of the source blockchain to std out.
pub fn run(config: Config) -> Result<(), Box<Error>> {
    let ethereum = blockchain::new_ethereum(config.source_address);
    let accounts = ethereum.get_accounts();

    println!("Accounts:");
    for account in accounts {
        println!("{}", account);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_config_reads_the_environment_variables() {
        let config = Config::new().unwrap();
        assert_eq!(config.source_address, DEFAULT_SOURCE_ADDRESS.to_string());
        assert_eq!(config.target_address, DEFAULT_TARGET_ADDRESS.to_string());

        env::set_var(ENV_SOURCE_ADDRESS, "10.0.0.1");
        let config = Config::new().unwrap();
        assert_eq!(config.source_address, "10.0.0.1");
        assert_eq!(config.target_address, DEFAULT_TARGET_ADDRESS.to_string());

        env::set_var(ENV_TARGET_ADDRESS, "10.0.0.2");
        let config = Config::new().unwrap();
        assert_eq!(config.source_address, "10.0.0.1");
        assert_eq!(config.target_address, "10.0.0.2");

        env::remove_var(ENV_SOURCE_ADDRESS);
        env::remove_var(ENV_TARGET_ADDRESS);
    }
}
