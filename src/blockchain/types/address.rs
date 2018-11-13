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

//! This module implements the Address struct and its methods.

use blockchain::types::error::{Error, ErrorKind};
use std::fmt;
use std::fmt::{Debug, Formatter, LowerHex};

/// An Address is represented by a 20-bytes address.
#[derive(PartialEq, Eq, Default, Clone, Copy)]
pub struct Address([u8; 20]);

impl Address {
    /// Creates an address with the address given as 20 bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 20 {
            return Err(Error::new(
                ErrorKind::InvalidAddress,
                format!(
                    "Address must have exactly 20 bytes. Got {} instead: {:?}",
                    bytes.len(),
                    bytes,
                ),
            ));
        }

        let mut bytes_array = [0u8; 20];
        for (index, byte) in bytes.into_iter().enumerate() {
            bytes_array[index] = *byte;
        }
        Ok(Address(bytes_array))
    }

    /// Creates an address from a string in hex format of the address.
    ///
    /// *Arguments*
    ///
    /// * `string` - A String in hex format that represents 20 bytes.
    ///              Must be exactly 40 characters long. Any leading `0x` will be removed.
    pub fn from_string(string: &str) -> Result<Self, Error> {
        let mut cleaned = &string.to_string()[..];
        cleaned = cleaned.trim();

        // cut leading "0x" if present
        if &cleaned[..2] == "0x" {
            cleaned = &cleaned[2..];
        }

        if cleaned.len() != 40 {
            return Err(Error::new(
                ErrorKind::InvalidAddress,
                format!(
                    "Expected 40 characters. Got {} instead: {}",
                    cleaned.len(),
                    cleaned
                ),
            ));
        }

        // Convert byte by byte.
        let mut bytes = [0u8; 20];
        let mut index = 0;
        loop {
            let byte = match u8::from_str_radix(&cleaned[..2], 16) {
                Ok(byte) => byte,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::InvalidAddress,
                        format!("Could not parse hex string into address bytes: {}", error),
                    ))
                }
            };

            bytes[index] = byte;

            cleaned = &cleaned[2..];
            if cleaned.len() < 2 {
                break;
            }
            index += 1;
        }

        Ok(Address(bytes))
    }

    /// Returns the bytes representation of this address.
    pub fn bytes(&self) -> [u8; 20] {
        self.0
    }
}

impl LowerHex for Address {
    /// Writes the bytes as hex with leading zeros to the given Formatter.
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        for byte in &self.bytes() {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl Debug for Address {
    /// Debug format is lower hex.
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:x}", self)?;

        Ok(())
    }
}

// Converting other types to and from Addresses
pub trait AsAddress {
    fn as_address(&self) -> Result<Address, Error>;
}

pub trait FromAddress {
    fn from_address(address: &Address) -> Self;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn address_from_string() {
        let mut address = Address::from_string("0000000000000000000000000000000000000000").unwrap();
        assert_eq!(
            format!("{:x}", address),
            "0000000000000000000000000000000000000000"
        );

        address = Address::from_string("0000000000000000000000000000000000000001").unwrap();
        assert_eq!(
            format!("{:x}", address),
            "0000000000000000000000000000000000000001"
        );

        address = Address::from_string("0x1000000000000000000000000000000000000000").unwrap();
        assert_eq!(
            format!("{:x}", address),
            "1000000000000000000000000000000000000000"
        );

        address = Address::from_string("0x123456789abcdef01234123456789abcdef01234").unwrap();
        assert_eq!(
            format!("{:x}", address),
            "123456789abcdef01234123456789abcdef01234"
        );

        address = Address::from_string("0x123456789ABCDEF01234123456789abcdef01234").unwrap();
        assert_eq!(
            format!("{:x}", address),
            "123456789abcdef01234123456789abcdef01234"
        );

        let mut result = Address::from_string("0x123456789ABCDEF01234123456789abcdef");
        assert!(result.is_err());
        result = Address::from_string("0x123456789ABCDEF01234123456789abcdef012341234");
        assert!(result.is_err());
    }

    #[test]
    fn address_to_lower_hex() {
        let mut bytes = [0u8; 20];
        let address = Address::from_bytes(&bytes[..]).unwrap();
        assert_eq!(
            format!("{:x}", address),
            "0000000000000000000000000000000000000000"
        );

        bytes[0] = 1u8;
        let address = Address::from_bytes(&bytes[..]).unwrap();
        assert_eq!(
            format!("{:x}", address),
            "0100000000000000000000000000000000000000"
        );

        bytes[19] = 18u8;
        let address = Address::from_bytes(&bytes[..]).unwrap();
        assert_eq!(
            format!("{:x}", address),
            "0100000000000000000000000000000000000012"
        );
    }

    #[test]
    fn equality() {
        let bytes = [4u8; 20];
        let address_one = Address::from_bytes(&bytes[..]).unwrap();

        let bytes = [4u8; 20];
        let address_two = Address::from_bytes(&bytes[..]).unwrap();

        let bytes = [5u8; 20];
        let address_three = Address::from_bytes(&bytes[..]).unwrap();

        assert!(address_one == address_two);
        assert!(address_one != address_three);
    }
}
