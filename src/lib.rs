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

use std::env;
use std::error::Error;

mod blockchain;

const ENV_VALUE_ADDRESS: &str = "MOSAIC_VALUE_ADDRESS";
const ENV_UTILITY_ADDRESS: &str = "MOSAIC_UTILITY_ADDRESS";
const DEFAULT_VALUE_ADDRESS: &str = "127.0.0.1:8485";
const DEFAULT_UTILITY_ADDRESS: &str = "127.0.0.1:8486";

/// Global config for running a mosaic node.
pub struct Config {
    /// Address of the value chain, e.g. "127.0.0.1:8485"
    value: String,
    /// Address of the utility chain, e.g. "127.0.0.1:8486"
    utility: String,
}

impl Config {
    /// Reads the configuration from environment variables and creates a new Config from them. In
    /// case an environment variable is not set, a default fallback will be used.
    pub fn new() -> Result<Config, &'static str> {
        // Read value address from env and set it or fallback to default
        let value = env::var(ENV_VALUE_ADDRESS);
        let value = match value {
            Ok(address) => address,
            Err(_) => {
                info!("No value chain address given, falling back to default.");
                DEFAULT_VALUE_ADDRESS.to_string()
            }
        };

        // Read utility address from env and set it or fallback to default
        let utility = env::var(ENV_UTILITY_ADDRESS);
        let utility = match utility {
            Ok(address) => address,
            Err(_) => {
                info!("No utility chain address given, falling back to default.");
                DEFAULT_UTILITY_ADDRESS.to_string()
            }
        };

        info!("Using value chain address: {}", value);
        info!("Using utility chain address: {}", utility);

        Ok(Config { value, utility })
    }
}

/// Runs a mosaic node with the given configuration.
pub fn run(config: Config) -> Result<(), Box<Error>> {
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_config_reads_the_environment_variables() {
        let config = Config::new().unwrap();
        assert_eq!(config.value, DEFAULT_VALUE_ADDRESS.to_string());
        assert_eq!(config.utility, DEFAULT_UTILITY_ADDRESS.to_string());

        env::set_var(ENV_VALUE_ADDRESS, "10.0.0.1");
        let config = Config::new().unwrap();
        assert_eq!(config.value, "10.0.0.1");
        assert_eq!(config.utility, DEFAULT_UTILITY_ADDRESS.to_string());

        env::set_var(ENV_UTILITY_ADDRESS, "10.0.0.2");
        let config = Config::new().unwrap();
        assert_eq!(config.value, "10.0.0.1");
        assert_eq!(config.utility, "10.0.0.2");

        env::remove_var(ENV_VALUE_ADDRESS);
        env::remove_var(ENV_UTILITY_ADDRESS);
    }
}
