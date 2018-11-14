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

//! An address represents the address of an external account or contract on a blockchain.

use blockchain::types::error::{Error, ErrorKind};
use std::fmt::{self, Debug, Formatter, LowerHex};
use std::str::FromStr;

/// An Address is represented by an array of 20 `u8`-bytes.
#[derive(PartialEq, Eq, Default, Clone, Copy)]
pub struct Address(pub [u8; 20]);

impl Address {
    /// Returns the underlying bytes array of the `Address`.
    pub fn bytes(&self) -> [u8; 20] {
        self.0
    }
}

impl From<[u8; 20]> for Address {
    /// Converts an array of bytes into an `Address`.
    fn from(bytes: [u8; 20]) -> Address {
        Address { 0: bytes }
    }
}

impl From<Address> for [u8; 20] {
    /// Converts an `Address` to its underlying array of bytes.
    fn from(address: Address) -> [u8; 20] {
        address.bytes()
    }
}

impl FromStr for Address {
    type Err = Error;

    /// Parses a string into an `Address`.
    /// Returns either an `Address` or an error if the string could not be converted.
    ///
    /// If the string has leading `0x`, then that is automatically removed. Afterwards, the string
    /// must be exactly 40 characters long and each pair of two characters must be parsable from hex
    /// into a `u8`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cleaned = s;
        cleaned = cleaned.trim();

        // Cut leading "0x" if present.
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

        Ok(bytes.into())
    }
}

impl LowerHex for Address {
    /// Writes the bytes as hex with leading zeros to the given Formatter.
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        for byte in self.bytes().iter() {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl Debug for Address {
    /// Debug format equals lower hex.
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:x}", self)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn address_from_string() {
        let mut address = "0000000000000000000000000000000000000000"
            .parse::<Address>()
            .unwrap();
        assert_eq!(
            format!("{:x}", address),
            "0000000000000000000000000000000000000000"
        );

        address = "0000000000000000000000000000000000000001"
            .parse::<Address>()
            .unwrap();
        assert_eq!(
            format!("{:x}", address),
            "0000000000000000000000000000000000000001"
        );

        address = "0x1000000000000000000000000000000000000000"
            .parse::<Address>()
            .unwrap();
        assert_eq!(
            format!("{:x}", address),
            "1000000000000000000000000000000000000000"
        );

        address = "0x123456789abcdef01234123456789abcdef01234"
            .parse::<Address>()
            .unwrap();
        assert_eq!(
            format!("{:x}", address),
            "123456789abcdef01234123456789abcdef01234"
        );

        address = "0x123456789ABCDEF01234123456789abcdef01234"
            .parse::<Address>()
            .unwrap();
        assert_eq!(
            format!("{:x}", address),
            "123456789abcdef01234123456789abcdef01234"
        );

        let mut result = "0x123456789ABCDEF01234123456789abcdef".parse::<Address>();
        assert!(result.is_err());
        result = "0x123456789ABCDEF01234123456789abcdef012341234".parse::<Address>();
        assert!(result.is_err());
    }

    #[test]
    fn address_to_lower_hex() {
        let mut bytes = [0u8; 20];
        let address: Address = bytes.into();
        assert_eq!(
            format!("{:x}", address),
            "0000000000000000000000000000000000000000"
        );

        bytes[0] = 1u8;
        let address: Address = bytes.into();
        assert_eq!(
            format!("{:x}", address),
            "0100000000000000000000000000000000000000"
        );

        bytes[19] = 18u8;
        let address: Address = bytes.into();
        assert_eq!(
            format!("{:x}", address),
            "0100000000000000000000000000000000000012"
        );
    }

    #[test]
    fn equality() {
        let bytes = [4u8; 20];
        let address_one: Address = bytes.into();

        let bytes = [4u8; 20];
        let address_two: Address = bytes.into();

        let bytes = [5u8; 20];
        let address_three: Address = bytes.into();

        assert!(address_one == address_two);
        assert!(address_one != address_three);
    }
}
