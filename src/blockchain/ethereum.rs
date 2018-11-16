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

use blockchain::*;
use rpassword;
use std::time::Duration;
use web3::transports::Http;
use web3::types::Block as Web3Block;
use web3::types::H256 as Web3H256;
use web3::types::U128 as Web3U128;
use web3::types::U256 as Web3U256;
use web3::types::{BlockId, H160, H520};
use web3::contract::Contract;
use web3::Web3;

/// This struct stores a connection to an Ethereum node.
#[derive(Clone)]
pub struct Ethereum {
    web3: Web3<Http>,
    validator: Address,
    password: String,
    event_loop: Box<tokio_core::reactor::Handle>,
}

trait IntoBlock {
    fn into_block(&self) -> Result<Block, Error>;
}

impl Ethereum {
    /// Creates a new instance of Ethereum pointing to the given address.
    /// Reads the password to unlock the account in the ethereum node from `stdin`.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of an ethereum node.
    /// * `validator` - The address of the validator to sign and send messages from.
    pub fn new(
        address: &str,
        validator: Address,
        event_loop: Box<tokio_core::reactor::Handle>,
    ) -> Self {
        let http = Http::with_event_loop(address, &event_loop, 5)
            .expect("Could not initialize ethereum HTTP connection");
        let web3 = Web3::new(http);

        let password = rpassword::prompt_password_stdout(&format!(
            "Please enter the password for account {:x}: ",
            &validator,
        )).unwrap();

        Ethereum {
            web3,
            validator,
            password,
            event_loop,
        }
    }

    /// Stream blocks returns a `futures::stream::Stream` of `Block`s.
    ///
    /// Converts a stream of web3 blocks to a stream of blocks.
    ///
    /// It is the caller's responsibility to poll the stream, e.g. call `for_each` and put the
    /// future into a reactor.
    pub fn stream_blocks(&self) -> impl Stream<Item = Block, Error = Error> {
        // Blocks filter is a future that returns a filter.
        let blocks_filter = self.web3.eth_filter().create_blocks_filter();

        // Block hashes is a stream of block hashes.
        let block_hashes = blocks_filter
            .map(|filter| filter.stream(Duration::from_secs(1)))
            .flatten_stream();

        // Web3 blocks is a stream of block futures, mapped from a stream of block hashes.
        let web3_clone = self.web3.clone();
        let web3_blocks = block_hashes
            .map_err(|error| {
                Error::new(
                    ErrorKind::NodeError,
                    format!("Error while streaming blocks from node: {}", error),
                )
            }).and_then(move |block_hash| {
                web3_clone
                    .eth()
                    .block(BlockId::from(block_hash))
                    .map_err(|error| {
                        Error::new(
                            ErrorKind::NodeError,
                            format!("Was not able to retrieve block: {}", error),
                        )
                    })
            });

        // Returns a stream of blocks, mapped from a stream of web3 block futures.
        web3_blocks.and_then(|web3_block| match web3_block {
            // Mapping web3 block Option to a Block.
            // Wrapping in Ok() as it has to return an IntoFuture.
            Some(web3_block) => match web3_block.into_block() {
                Ok(block) => Ok(block),
                Err(error) => Err(Error::new(
                    ErrorKind::NodeError,
                    format!("Could not convert block from web3: {}", error),
                )),
            },
            None => Err(Error::new(
                ErrorKind::NodeError,
                "No block found".to_string(),
            )),
        })
    }

    /// Uses web3 to retrieve the accounts.
    /// Converts them to blockchain addresses and returns all addresses in a
    /// vector.
    pub fn get_accounts(&self) -> impl Future<Item = Vec<Address>, Error = Error> {
        self.web3
            .eth()
            .accounts()
            .map_err(|error| {
                Error::new(
                    ErrorKind::NodeError,
                    format!("Was not able to retrieve accounts: {}", error),
                )
            }).and_then(|addresses| {
                let mut v = Vec::new();
                for h160 in addresses {
                    v.push(h160.into())
                }

                Ok(v)
            })
    }

    /// Uses web3 to sign the given data.
    /// Converts the signature to a blockchain signature.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to sign.
    ///
    /// # Returns
    ///
    /// Returns a `Signature` of the signed data.
    pub fn sign(&self, data: Bytes) -> impl Future<Item = Signature, Error = Error> {
        let web3_clone = self.web3.clone();
        let validator = self.validator;

        let signature_future = self.unlock_account(None).and_then(move |_| {
            web3_clone
                .eth()
                .sign(validator.into(), web3::types::Bytes(data.bytes().clone()))
                .map_err(|error| {
                    Error::new(
                        ErrorKind::NodeError,
                        format!("Was not able to sign data: {}", error),
                    )
                })
        });

        signature_future.map(|web3_signature| {
            let signature: Signature = web3_signature.into();
            signature
        })
    }

    /// Create contract instance
    ///
    /// # Arguments
    ///
    /// * `contract_address` -  The address of contract.
    /// * `abi` - ABI of contract.
    ///
    /// # Returns
    ///
    /// Returns a `contract` instance.
    pub fn contract_instance(&self, contract_address: Address, abi: &[u8]) -> Contract<Http> {
        Contract::from_json(
            self.web3.eth(),
            H160::from(contract_address),
            abi,
        ).unwrap()
    }

    /// Unlocks the validator account of this ethereum instance using the stored password.
    ///
    /// # Arguments
    ///
    /// * `duration` - If given, will unlock for the duration in seconds. Otherwise for a single
    /// transaction.
    ///
    /// # Panics
    ///
    /// Panics if it cannot unlock the account.
    fn unlock_account(&self, duration: Option<u16>) -> impl Future<Item = bool, Error = Error> {
        self.web3
            .personal()
            .unlock_account(self.validator.into(), &self.password, duration)
            .map_err(|error| {
                Error::new(
                    ErrorKind::NodeError,
                    format!("Was not able to unlock account: {}", error),
                )
            })
    }
}

impl From<Address> for H160 {
    /// Creates an H160 type from an address.
    /// H160 will equal the address's bytes.
    fn from(address: Address) -> H160 {
        let bytes: [u8; 20] = address.into();
        H160::from(bytes)
    }
}

impl From<H160> for Address {
    /// Converts an H160 type to an Address.
    /// The address's bytes will be a copy of H160.
    fn from(h160: H160) -> Address {
        h160.0.into()
    }
}

impl From<Web3H256> for H256 {
    /// Converts a web3 H256 into an `H256`.
    fn from(h256: Web3H256) -> H256 {
        h256.0.into()
    }
}

impl From<Web3U128> for U128 {
    /// Converts a web3 U128 into a `U128`.
    fn from(u128: Web3U128) -> U128 {
        u128.0.into()
    }
}

impl From<Web3U256> for U256 {
    /// Converts a web3 U256 into a `U256`.
    fn from(u256: Web3U256) -> U256 {
        u256.0.into()
    }
}

impl From<H520> for Signature {
    /// Converts a web3 H520 into a `Signature`.
    fn from(h520: H520) -> Signature {
        Signature { 0: h520.0 }
    }
}

impl<TX> IntoBlock for Web3Block<TX> {
    /// Tries to convert a web3 block into a `Block`.
    ///
    /// Fails if mandatory fields are missing.
    fn into_block(&self) -> Result<Block, Error> {
        Ok(Block {
            hash: match self.hash {
                Some(hash) => hash.into(),
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidBlock,
                        "Block has no hash".to_string(),
                    ));
                }
            },
            parent_hash: self.parent_hash.into(),
            state_root: self.state_root.into(),
            transactions_root: self.transactions_root.into(),
            number: match self.number {
                Some(number) => number.into(),
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidBlock,
                        "Block has no number".to_string(),
                    ))
                }
            },
            gas_used: self.gas_used.into(),
            gas_limit: self.gas_limit.into(),
            timestamp: self.timestamp.into(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_h160_to_address() {
        let mut bytes = [0u8; 20];
        let address: Address = "0000000000000000000000000000000000000000"
            .parse::<H160>()
            .unwrap()
            .into();
        assert_eq!(address, bytes.into());

        bytes[19] = 10u8;
        let address: Address = "000000000000000000000000000000000000000a"
            .parse::<H160>()
            .unwrap()
            .into();
        assert_eq!(address, bytes.into());

        bytes[0] = 1u8;
        let address: Address = "010000000000000000000000000000000000000a"
            .parse::<H160>()
            .unwrap()
            .into();
        assert_eq!(address, bytes.into());
    }

    #[test]
    fn test_h160_from_address() {
        let mut bytes = [0u8; 20];
        let address: Address = bytes.into();
        let h160: H160 = address.into();
        assert_eq!(
            format!("{:#?}", h160),
            "0x0000000000000000000000000000000000000000"
        );

        bytes[19] = 10u8;
        let address: Address = bytes.into();
        let h160: H160 = address.into();
        assert_eq!(
            format!("{:#?}", h160),
            "0x000000000000000000000000000000000000000a"
        );

        bytes[0] = 1u8;
        let address: Address = bytes.into();
        let h160: H160 = address.into();
        assert_eq!(
            format!("{:#?}", h160),
            "0x010000000000000000000000000000000000000a"
        );
    }
}
