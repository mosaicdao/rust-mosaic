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

use blockchain::Address;
use std::env;

// Environment variables and their defaults
const ENV_ORIGIN_ENDPOINT: &str = "MOSAIC_ORIGIN_ENDPOINT";
const DEFAULT_ORIGIN_ENDPOINT: &str = "http://127.0.0.1:8545";
const ENV_AUXILIARY_ENDPOINT: &str = "MOSAIC_AUXILIARY_ENDPOINT";
const DEFAULT_AUXILIARY_ENDPOINT: &str = "http://127.0.0.1:8546";
const ENV_ORIGIN_CORE_ADDRESS: &str = "MOSAIC_ORIGIN_CORE_ADDRESS";
const ENV_ORIGIN_VALIDATOR_ADDRESS: &str = "MOSAIC_ORIGIN_VALIDATOR_ADDRESS";
const ENV_AUXILIARY_VALIDATOR_ADDRESS: &str = "MOSAIC_AUXILIARY_VALIDATOR_ADDRESS";
const ENV_ORIGIN_BLOCK_STORE_ADDRESS: &str = "MOSAIC_ORIGIN_BLOCK_STORE_ADDRESS";

/// Global config for running a mosaic node.
#[derive(Default)]
pub struct Config {
    /// Address of the origin chain, e.g. "127.0.0.1:8485"
    origin_endpoint: String,
    /// Address of the auxiliary chain, e.g. "127.0.0.1:8486"
    auxiliary_endpoint: String,
    /// The address of a core address on origin.
    /// It is optional as it may not be needed depending on the mode that the node is run in.
    _origin_core_address: Option<Address>,
    /// The address that is used to send messages as a validator on origin.
    origin_validator_address: Address,
    /// The address that is used to send messages as a validator on auxiliary.
    auxiliary_validator_address: Address,
    /// The address of origin block store contract.
    origin_block_store_address: Address,
}

impl Config {
    /// Reads the configuration from environment variables and creates a new Config from them. In
    /// case an environment variable is not set, a default fallback will be used if available.
    ///
    /// # Returns
    ///
    /// Returns a configuration with the settings read from the environment.
    ///
    /// # Panics
    ///
    /// This function panics if a mandatory value cannot be read and there is no default.
    /// This function also panics if a value cannot be parsed into its appropriate type.
    pub fn new() -> Config {
        let origin_endpoint = match Self::read_environment_variable(
            ENV_ORIGIN_ENDPOINT,
            Some(DEFAULT_ORIGIN_ENDPOINT),
        ) {
            Some(origin_endpoint) => origin_endpoint,
            None => panic!("An origin endpoint must be set"),
        };
        let auxiliary_endpoint = match Self::read_environment_variable(
            ENV_AUXILIARY_ENDPOINT,
            Some(DEFAULT_AUXILIARY_ENDPOINT),
        ) {
            Some(auxiliary_endpoint) => auxiliary_endpoint,
            None => panic!("An auxiliary endpoint must be set"),
        };

        let origin_core_address =
            match Self::read_environment_variable(ENV_ORIGIN_CORE_ADDRESS, None) {
                Some(origin_core_address) => Some(
                    origin_core_address
                        .parse::<Address>()
                        .expect("The origin core address cannot be parsed"),
                ),
                None => None,
            };

        let origin_validator_address =
            match Self::read_environment_variable(ENV_ORIGIN_VALIDATOR_ADDRESS, None) {
                Some(origin_validator_address) => origin_validator_address
                    .parse::<Address>()
                    .expect("The origin validator address cannot be parsed"),
                None => panic!("An origin validator address must be set"),
            };

        let auxiliary_validator_address =
            match Self::read_environment_variable(ENV_AUXILIARY_VALIDATOR_ADDRESS, None) {
                Some(auxiliary_validator_address) => auxiliary_validator_address
                    .parse::<Address>()
                    .expect("The auxiliary validator address cannot be parsed"),
                None => panic!("An auxiliary validator address must be set"),
            };

        let origin_block_store_address =
            match Self::read_environment_variable(ENV_ORIGIN_BLOCK_STORE_ADDRESS, None) {
                Some(auxiliary_validator_address) => auxiliary_validator_address
                    .parse::<Address>()
                    .expect("The origin block store address cannot be parsed"),
                None => panic!("An origin block store address must be set"),
            };

        Config {
            origin_endpoint,
            auxiliary_endpoint,
            _origin_core_address: origin_core_address,
            origin_validator_address,
            auxiliary_validator_address,
            origin_block_store_address
        }
    }

    /// Reads an environment variable and return the value if found or a default if given.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the environment variable.
    /// * `default_value` - An optional default value if the environment variable is not set.
    ///
    /// # Returns
    ///
    /// An optional string that is the value of the environment variable if set or the default if
    /// given.
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

    /// Returns the origin endpoint set on this config.
    pub fn origin_endpoint(&self) -> &String {
        &self.origin_endpoint
    }

    /// Returns the auxiliary endpoint set on this config.
    pub fn auxiliary_endpoint(&self) -> &String {
        &self.auxiliary_endpoint
    }

    /// Returns the origin validator address set on this config.
    pub fn origin_validator_address(&self) -> Address {
        self.origin_validator_address
    }

    /// Returns the auxiliary validator address set on this config.
    pub fn auxiliary_validator_address(&self) -> Address {
        self.auxiliary_validator_address
    }

    /// Returns the address of origin block store.
    pub fn origin_block_store_address(&self) -> Address {
        self.origin_block_store_address
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_config_reads_the_environment_variables() {
        env::set_var(
            ENV_ORIGIN_VALIDATOR_ADDRESS,
            "6789012345678901234567890123456789012345",
        );
        env::set_var(
            ENV_AUXILIARY_VALIDATOR_ADDRESS,
            "1234567890123456789012345678901234567890",
        );
        let expected_origin_endpoint = "10.0.0.1";
        env::set_var(ENV_ORIGIN_ENDPOINT, expected_origin_endpoint);

        let config = Config::new();
        assert_eq!(
            config.origin_endpoint, expected_origin_endpoint,
            "Did not read the origin endpoint {}, but {} instead",
            expected_origin_endpoint, config.origin_endpoint,
        );
        assert_eq!(
            config.origin_validator_address(),
            "6789012345678901234567890123456789012345"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            config.auxiliary_validator_address(),
            "1234567890123456789012345678901234567890"
                .parse::<Address>()
                .unwrap()
        );

        env::set_var(ENV_ORIGIN_ENDPOINT, "10.0.0.1");
        let config = Config::new();
        assert_eq!(config.origin_endpoint, "10.0.0.1");
        // Assert also that it does not overwrite the wrong configuration value.
        assert_eq!(
            config.auxiliary_endpoint,
            DEFAULT_AUXILIARY_ENDPOINT.to_owned()
        );

        let expected_auxiliary_endpoint = "10.0.0.2";
        env::set_var(ENV_AUXILIARY_ENDPOINT, expected_auxiliary_endpoint);
        let config = Config::new();
        assert_eq!(
            config.origin_endpoint, expected_origin_endpoint,
            "Did not read the origin endpoint {}, but {} instead",
            expected_origin_endpoint, config.origin_endpoint,
        );
        assert_eq!(
            config.auxiliary_endpoint, expected_auxiliary_endpoint,
            "Did not read the auxiliary endpoint {}, but {} instead",
            expected_auxiliary_endpoint, config.auxiliary_endpoint,
        );

        env::remove_var(ENV_ORIGIN_ENDPOINT);
        env::remove_var(ENV_AUXILIARY_ENDPOINT);
        env::remove_var(ENV_ORIGIN_VALIDATOR_ADDRESS);
        env::remove_var(ENV_AUXILIARY_VALIDATOR_ADDRESS);
    }

    #[test]
    fn the_config_falls_back_to_the_default() {
        // These must be set without a fallback
        env::set_var(
            ENV_ORIGIN_VALIDATOR_ADDRESS,
            "6789012345678901234567890123456789012345",
        );
        env::set_var(
            ENV_AUXILIARY_VALIDATOR_ADDRESS,
            "1234567890123456789012345678901234567890",
        );

        let config = Config::new();
        assert_eq!(
            config.origin_endpoint,
            DEFAULT_ORIGIN_ENDPOINT.to_owned(),
            "Did not set the default origin endpoint when no ENV var set.",
        );
        assert_eq!(
            config.auxiliary_endpoint,
            DEFAULT_AUXILIARY_ENDPOINT.to_owned(),
            "Did not set the default auxiliary endpoint when no ENV var set.",
        );

        env::remove_var(ENV_ORIGIN_VALIDATOR_ADDRESS);
        env::remove_var(ENV_AUXILIARY_VALIDATOR_ADDRESS);
    }
}
