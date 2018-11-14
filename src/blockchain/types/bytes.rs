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

//! This module implements the Bytes struct and its methods.

use blockchain::types::error::{Error, ErrorKind};
use std::str::FromStr;

/// A wrapper for bytes.
#[derive(PartialEq, Eq, Default, Clone, Debug)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    /// Returns the underlying `u8` vector of this `Bytes` object.
    pub fn bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
}

impl From<Vec<u8>> for Bytes {
    /// Converts a vector of `u8`s into a `Bytes` object.
    fn from(vector: Vec<u8>) -> Bytes {
        Bytes { 0: vector }
    }
}

impl FromStr for Bytes {
    type Err = Error;

    /// Parses a string into a `Bytes` object.
    /// Any leading `0x` is automatically removed.
    ///
    /// The string must have an even length and each pair of two characters must be parsable from
    /// hex into `u8`.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut cleaned = &string[..];
        cleaned = cleaned.trim();

        // Cut leading "0x" if present.
        if &cleaned[..2] == "0x" {
            cleaned = &cleaned[2..];
        }

        // Empty string leads to empty vector.
        if cleaned.is_empty() {
            return Ok(Bytes(vec![]));
        }

        if cleaned.len() % 2 != 0 {
            return Err(Error::new(
                ErrorKind::InvalidBytes,
                "String must be a multiple of two characters long to be converted to Bytes"
                    .to_string(),
            ));
        }

        // Convert byte by byte.
        let mut bytes: Vec<u8> = Vec::new();
        loop {
            let byte = match u8::from_str_radix(&cleaned[..2], 16) {
                Ok(byte) => byte,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::InvalidBytes,
                        format!("Could not parse hex string into bytes: {}", error),
                    ))
                }
            };

            bytes.push(byte);

            cleaned = &cleaned[2..];
            if cleaned.len() < 2 {
                break;
            }
        }

        Ok(Bytes(bytes))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bytes_from_string() {
        let mut bytes = "0000000000000000000000000000000000000000"
            .parse::<Bytes>()
            .unwrap();
        assert_eq!(
            format!("{:?}", bytes),
            "Bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])"
        );

        bytes = "0000000000000000000000000000000000000001"
            .parse::<Bytes>()
            .unwrap();
        assert_eq!(
            format!("{:?}", bytes),
            "Bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1])"
        );

        bytes = "0x1000000000000000000000000000000000000000"
            .parse::<Bytes>()
            .unwrap();
        assert_eq!(
            format!("{:?}", bytes),
            "Bytes([16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])"
        );

        bytes = "0x123456789abcdef01234123456789abcdef01234"
            .parse::<Bytes>()
            .unwrap();
        assert_eq!(
            format!("{:?}", bytes),
            "Bytes([18, 52, 86, 120, 154, 188, 222, 240, 18, 52, 18, 52, 86, 120, 154, 188, 222, 240, 18, 52])"
        );

        bytes = "0x123456789ABCDEF01234123456789abcdef01234"
            .parse::<Bytes>()
            .unwrap();
        assert_eq!(
            format!("{:?}", bytes),
            "Bytes([18, 52, 86, 120, 154, 188, 222, 240, 18, 52, 18, 52, 86, 120, 154, 188, 222, 240, 18, 52])"
        );

        let mut result = "0x123456789ABCDEF01234123456789abcdef".parse::<Bytes>();
        assert!(result.is_err());
        result = "0x123456789ABCDEF01234123456789abcdef0123412345".parse::<Bytes>();
        assert!(result.is_err());
        result = "0x123456789ABCDEF01234123456789abcdef01234123k".parse::<Bytes>();
        assert!(result.is_err());
    }

    #[test]
    fn equality() {
        let bytes_one = "0000000000000000000000000000000000000012"
            .parse::<Bytes>()
            .unwrap();
        let bytes_two = "0000000000000000000000000000000000000012"
            .parse::<Bytes>()
            .unwrap();
        let bytes_three = "0000000000000000000000000000000000000034"
            .parse::<Bytes>()
            .unwrap();
        let bytes_four = "00000000000000000000000000000000000012"
            .parse::<Bytes>()
            .unwrap();

        assert!(bytes_one == bytes_two);
        assert!(bytes_one != bytes_three);
        assert!(bytes_one != bytes_four);
    }
}
