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

use std::fmt::{self, Debug, Formatter, LowerHex};

/// A Signature is represented by 65-bytes.
#[derive(Clone, Copy)]
pub struct Signature(pub [u8; 65]);

impl Signature {
    /// Returns the underlying `u8` array of a `Signature`.
    pub fn bytes(&self) -> [u8; 65] {
        self.0
    }
}

impl From<[u8; 65]> for Signature {
    /// Converts a `u8` array of 65 elements into a `Signature`.
    fn from(bytes: [u8; 65]) -> Self {
        Self { 0: bytes }
    }
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
    /// Two `Signature`s are equal, if the underlying arrays are equal.
    fn eq(&self, other: &Signature) -> bool {
        for (index, byte) in self.bytes().iter().enumerate() {
            if byte != &other.bytes()[index] {
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
        let signature: Signature = bytes.into();
        assert_eq!(
            format!("{:x}", signature),
            "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );

        bytes[0] = 1u8;
        let signature: Signature = bytes.into();
        assert_eq!(
            format!("{:x}", signature),
            "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );

        bytes[19] = 18u8;
        let signature: Signature = bytes.into();
        assert_eq!(
            format!("{:x}", signature),
            "0100000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn equality() {
        let bytes = [4u8; 65];
        let signature_one: Signature = bytes.into();

        let bytes = [4u8; 65];
        let signature_two: Signature = bytes.into();

        let bytes = [5u8; 65];
        let signature_three: Signature = bytes.into();

        assert!(signature_one == signature_two);
        assert!(signature_one != signature_three);
    }
}
