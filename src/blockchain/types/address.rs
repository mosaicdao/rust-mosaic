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

use std::cmp::PartialEq;
use std::fmt::{Debug, Error, Formatter, LowerHex};

/// An Address is represented by a 20-bytes address.
pub struct Address([u8; 20]);

impl Address {
    /// Creates an address with the address given as 20 bytes.
    pub fn new(bytes: [u8; 20]) -> Self {
        Address(bytes)
    }

    /// Returns the bytes representation of this address.
    pub fn bytes(&self) -> [u8; 20] {
        self.0
    }
}

impl PartialEq for Address {
    /// Two addresss are equal if their byte representations are equal.
    fn eq(&self, other: &Address) -> bool {
        self.bytes() == other.bytes()
    }
}

impl LowerHex for Address {
    /// Writes the bytes as hex with leading zeros to the given Formatter.
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl Debug for Address {
    /// Debug format is lower hex.
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{:x}", self)?;

        Ok(())
    }
}

// Converting other types to and from Addresss
pub trait AsAddress {
    fn as_address(&self) -> Address;
}

pub trait FromAddress {
    fn from_address(address: Address) -> Self;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn address_to_lower_hex() {
        let mut bytes = [0u8; 20];
        let address = Address::new(bytes);
        assert_eq!(
            format!("{:x}", address),
            "0000000000000000000000000000000000000000"
        );

        bytes[0] = 1u8;
        let address = Address::new(bytes);
        assert_eq!(
            format!("{:x}", address),
            "0100000000000000000000000000000000000000"
        );

        bytes[19] = 18u8;
        let address = Address::new(bytes);
        assert_eq!(
            format!("{:x}", address),
            "0100000000000000000000000000000000000012"
        );
    }
}
