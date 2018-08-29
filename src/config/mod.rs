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

//! This module handles all configuration of this library.

use blockchain::types::address::Address;
use std::env;

// Environment variables and their defaults
const ENV_ORIGIN_ENDPOINT: &str = "MOSAIC_ORIGIN_ENDPOINT";
const DEFAULT_ORIGIN_ENDPOINT: &str = "http://127.0.0.1:8545";
const ENV_AUXILIARY_ENDPOINT: &str = "MOSAIC_AUXILIARY_ENDPOINT";
const DEFAULT_AUXILIARY_ENDPOINT: &str = "http://127.0.0.1:8546";
const ENV_ORIGIN_CORE_ADDRESS: &str = "MOSAIC_ORIGIN_CORE_ADDRESS";

/// Global config for running a mosaic node.
#[derive(Default)]
pub struct Config {
    /// Address of the origin chain, e.g. "127.0.0.1:8485"
    origin_endpoint: String,
    /// Address of the auxiliary chain, e.g. "127.0.0.1:8486"
    _auxiliary_endpoint: String,
    /// The address of a core address on origin.
    /// It is optional as it may not be needed depending on the mode that the node is run in.
    _origin_core_address: Option<Address>,
}

impl Config {
    /// Reads the configuration from environment variables and creates a new Config from them. In
    /// case an environment variable is not set, a default fallback will be used.
    pub fn new() -> Config {
        let origin_endpoint =
            Self::read_environment_variable(ENV_ORIGIN_ENDPOINT, Some(DEFAULT_ORIGIN_ENDPOINT));
        let auxiliary_endpoint = Self::read_environment_variable(
            ENV_AUXILIARY_ENDPOINT,
            Some(DEFAULT_AUXILIARY_ENDPOINT),
        );

        let origin_core_address = match Self::read_environment_variable(
            ENV_ORIGIN_CORE_ADDRESS,
            None,
        ) {
            Some(origin_core_address) => Some(Address::from_string(&origin_core_address).unwrap()),
            None => None,
        };

        Config {
            origin_endpoint: match origin_endpoint {
                Some(origin_endpoint) => origin_endpoint,
                None => panic!("An origin endpoint must be set!"),
            },
            _auxiliary_endpoint: match auxiliary_endpoint {
                Some(auxiliary_endpoint) => auxiliary_endpoint,
                None => panic!("An auxiliary endpoint must be set!"),
            },
            _origin_core_address: origin_core_address,
        }
    }

    fn read_environment_variable(name: &str, default_value: Option<&str>) -> Option<String> {
        let value = match env::var(name) {
            Ok(value) => Some(value),
            Err(_) => match default_value {
                Some(default_value) => {
                    info!("No {} found, falling back to default.", name);
                    Some(default_value.to_owned())
                }
                None => None,
            },
        };

        info!(
            "Using {}: {}",
            name,
            match &value {
                Some(value) => value,
                None => "<not set>",
            }
        );

        value
    }

    pub fn origin_endpoint(&self) -> &String {
        &self.origin_endpoint
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_config_reads_the_environment_variables() {
        let config = Config::new();
        assert_eq!(config.origin_endpoint, DEFAULT_ORIGIN_ENDPOINT.to_owned());
        assert_eq!(
            config._auxiliary_endpoint,
            DEFAULT_AUXILIARY_ENDPOINT.to_owned()
        );

        env::set_var(ENV_ORIGIN_ENDPOINT, "10.0.0.1");
        let config = Config::new();
        assert_eq!(config.origin_endpoint, "10.0.0.1");
        assert_eq!(
            config._auxiliary_endpoint,
            DEFAULT_AUXILIARY_ENDPOINT.to_owned()
        );

        env::set_var(ENV_AUXILIARY_ENDPOINT, "10.0.0.2");
        let config = Config::new();
        assert_eq!(config.origin_endpoint, "10.0.0.1");
        assert_eq!(config._auxiliary_endpoint, "10.0.0.2");

        env::remove_var(ENV_ORIGIN_ENDPOINT);
        env::remove_var(ENV_AUXILIARY_ENDPOINT);
    }
}
