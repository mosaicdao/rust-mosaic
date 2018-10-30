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

//! This module implements the Signature struct and its methods.

use blockchain::types::error::{Error, ErrorKind};
use std::fmt;
use std::fmt::{Debug, Formatter, LowerHex};

/// A Signature is represented by 65-bytes.
#[derive(Clone, Copy)]
pub struct Signature([u8; 65]);

impl Signature {
    /// Creates a signature from a slice of 65 bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 65 {
            return Err(Error::new(
                ErrorKind::InvalidSignature,
                format!(
                    "Signature must have exactly 65 bytes. Got {} instead: {:?}",
                    bytes.len(),
                    bytes,
                ),
            ));
        }

        let mut bytes_array = [0u8; 65];
        let mut index = 0;
        for byte in bytes {
            bytes_array[index] = *byte;
            index += 1;
        }

        Ok(Signature(bytes_array))
    }

    /// Returns the bytes representation of this signature.
    pub fn bytes(&self) -> [u8; 65] {
        self.0
    }
}

// Converting other types to a Signature
pub trait AsSignature {
    fn as_signature(&self) -> Result<Signature, Error>;
}

impl LowerHex for Signature {
    /// Writes the bytes as hex with leading zeros to the given Formatter.
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        for byte in self.bytes().iter() {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl Debug for Signature {
    /// Debug format is lower hex.
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:x}", self)?;

        Ok(())
    }
}

impl std::cmp::PartialEq for Signature {
    fn eq(&self, other: &Signature) -> bool {
        for (index, byte) in self.bytes().iter().enumerate() {
            if *byte != other.bytes()[index] {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn signature_from_bytes() {
        let mut bytes = [0u8; 65];
        let signature = Signature::from_bytes(&bytes[..]).unwrap();
        assert_eq!(
            format!("{:x}", signature),
            "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );

        bytes[0] = 1u8;
        let signature = Signature::from_bytes(&bytes[..]).unwrap();
        assert_eq!(
            format!("{:x}", signature),
            "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );

        bytes[19] = 18u8;
        let signature = Signature::from_bytes(&bytes[..]).unwrap();
        assert_eq!(
            format!("{:x}", signature),
            "0100000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );

        let short_bytes = [0u8; 64];
        let result = Signature::from_bytes(&short_bytes[..]);
        assert!(result.is_err());
        let long_bytes = [0u8; 66];
        let result = Signature::from_bytes(&long_bytes[..]);
        assert!(result.is_err());
    }

    #[test]
    fn equality() {
        let bytes = [4u8; 65];
        let signature_one = Signature::from_bytes(&bytes[..]).unwrap();

        let bytes = [4u8; 65];
        let signature_two = Signature::from_bytes(&bytes[..]).unwrap();

        let bytes = [5u8; 65];
        let address_three = Signature::from_bytes(&bytes[..]).unwrap();

        assert!(signature_one == signature_two);
        assert!(signature_one != address_three);
    }
}
