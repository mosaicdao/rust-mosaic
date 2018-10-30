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

//! This module implements the connection to an Ethereum blockchain.

use rpassword;
use web3::futures::Future;
use web3::transports::{EventLoopHandle, Http};
use web3::types::{H160, H520};
use web3::Web3;

use super::types::address::{Address, AsAddress, FromAddress};
use super::types::bytes::Bytes;
use super::types::error::Error;
use super::types::signature::{AsSignature, Signature};

/// This struct stores a connection to an Ethereum node.
pub struct Ethereum {
    web3: Web3<Http>,
    validator: Address,
    password: String,
    _event_loop: EventLoopHandle,
}

impl Ethereum {
    /// Creates a new instance of Ethereum pointing to the given address.
    /// Reads the password to unlock the account in the ethereum node from `stdin`.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of an ethereum node.
    /// * `validator` - The address of the validator to sign and send messages from.
    pub fn new(address: &str, validator: Address) -> Self {
        let (event_loop, http) = Http::new(address).unwrap();
        let web3 = Web3::new(http);

        let password = rpassword::prompt_password_stdout(&format!(
            "Please enter the password for account {:x}: ",
            &validator,
        )).unwrap();

        let ethereum = Ethereum {
            web3,
            validator,
            password,
            _event_loop: event_loop,
        };

        ethereum.unlock_account();

        ethereum
    }

    /// Uses web3 to retrieve the accounts.
    /// Converts them to blockchain addresses and returns all addresses in a
    /// vector.
    pub fn get_accounts(&self) -> Vec<Address> {
        let addresses = self.web3.eth().accounts().wait().unwrap();
        let mut v = Vec::new();

        for h160 in addresses {
            match h160.as_address() {
                Ok(address) => v.push(address),
                Err(error) => warn!("Unable to convert h160 to address: {}", error),
            }
        }

        v
    }

    /// Uses web3 to sign the given data.
    /// Converts the signature to a blockchain signature.
    ///
    /// # Arguments
    ///
    /// `data` - The data to sign.
    ///
    /// # Returns
    ///
    /// Returns a `Signature` of the signed data.
    pub fn sign(&self, data: Bytes) -> Result<Signature, Error> {
        let h520 = self
            .web3
            .eth()
            .sign(
                H160::from_address(&self.validator),
                web3::types::Bytes(data.bytes()),
            ).wait()
            .unwrap();

        h520.as_signature()
    }

    /// Unlocks the validator account of this ethereum instance using the stored password.
    /// Unlocks it for the maximum amount of ca. 18 hours.
    ///
    /// # Panics
    ///
    /// Panics if it cannot unlock the account.
    fn unlock_account(&self) {
        let duration: u16 = 65535;

        let unlocked = self
            .web3
            .personal()
            .unlock_account(
                H160::from_address(&self.validator),
                &self.password,
                Some(duration),
            ).wait()
            .expect("Could not unlock account on ethereum node");

        if unlocked {
            info!("Unlocked account {:x}", &self.validator);
        } else {
            panic!("Could not unlock account {:x}", &self.validator);
        }
    }
}

impl AsAddress for H160 {
    /// Converts an H160 type to an Address.
    /// The address's bytes will be a copy of H160.
    fn as_address(&self) -> Result<Address, Error> {
        Address::from_bytes(&self[..])
    }
}

impl FromAddress for H160 {
    /// Creates an H160 type from an address.
    /// H160 will equal the address's bytes.
    fn from_address(address: &Address) -> Self {
        H160::from(address.bytes())
    }
}

impl AsSignature for H520 {
    fn as_signature(&self) -> Result<Signature, Error> {
        Signature::from_bytes(&self[..])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_h160_to_address() {
        let mut bytes = [0u8; 20];
        assert_eq!(
            "0000000000000000000000000000000000000000"
                .parse::<H160>()
                .unwrap()
                .as_address()
                .unwrap(),
            Address::from_bytes(&bytes[..]).unwrap()
        );

        bytes[19] = 10u8;
        assert_eq!(
            "000000000000000000000000000000000000000a"
                .parse::<H160>()
                .unwrap()
                .as_address()
                .unwrap(),
            Address::from_bytes(&bytes[..]).unwrap()
        );

        bytes[0] = 1u8;
        assert_eq!(
            "010000000000000000000000000000000000000a"
                .parse::<H160>()
                .unwrap()
                .as_address()
                .unwrap(),
            Address::from_bytes(&bytes[..]).unwrap()
        );
    }

    #[test]
    fn test_h160_from_address() {
        let mut bytes = [0u8; 20];
        assert_eq!(
            format!(
                "{:#?}",
                H160::from_address(&Address::from_bytes(&bytes[..]).unwrap())
            ),
            "0x0000000000000000000000000000000000000000"
        );

        bytes[19] = 10u8;
        assert_eq!(
            format!(
                "{:#?}",
                H160::from_address(&Address::from_bytes(&bytes[..]).unwrap())
            ),
            "0x000000000000000000000000000000000000000a"
        );

        bytes[0] = 1u8;
        assert_eq!(
            format!(
                "{:#?}",
                H160::from_address(&Address::from_bytes(&bytes[..]).unwrap())
            ),
            "0x010000000000000000000000000000000000000a"
        );
    }
}
