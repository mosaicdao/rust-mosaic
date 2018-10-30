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

/// A wrapper for bytes.
#[derive(PartialEq, Eq, Default, Clone, Debug)]
pub struct Bytes(Vec<u8>);

impl Bytes {
    /// Creates a bytes object from a string in hex format.
    ///
    /// *Arguments*
    ///
    /// * `string` - A String in hex format that represents bytes.
    ///              Must be at least two characters long and a multiple of two.
    ///              Any leading `0x` will be removed.
    pub fn from_string(string: &str) -> Result<Self, Error> {
        if string.len() < 2 {
            return Err(Error::new(
                ErrorKind::InvalidBytes,
                "String must be at least two characters long to be converted to Bytes".to_string(),
            ));
        }

        if string.len() % 2 != 0 {
            return Err(Error::new(
                ErrorKind::InvalidBytes,
                "String must be a multiple of two characters long to be converted to Bytes"
                    .to_string(),
            ));
        }

        let mut cleaned = &string[..];
        cleaned = cleaned.trim();

        // cut leading "0x" if present
        if &cleaned[..2] == "0x" {
            cleaned = &cleaned[2..];
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

    /// Returns the vector representation of these.
    pub fn bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bytes_from_string() {
        let mut bytes = Bytes::from_string("0000000000000000000000000000000000000000").unwrap();
        assert_eq!(
            format!("{:?}", bytes),
            "Bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])"
        );

        bytes = Bytes::from_string("0000000000000000000000000000000000000001").unwrap();
        assert_eq!(
            format!("{:?}", bytes),
            "Bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1])"
        );

        bytes = Bytes::from_string("0x1000000000000000000000000000000000000000").unwrap();
        assert_eq!(
            format!("{:?}", bytes),
            "Bytes([16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])"
        );

        bytes = Bytes::from_string("0x123456789abcdef01234123456789abcdef01234").unwrap();
        assert_eq!(
            format!("{:?}", bytes),
            "Bytes([18, 52, 86, 120, 154, 188, 222, 240, 18, 52, 18, 52, 86, 120, 154, 188, 222, 240, 18, 52])"
        );

        bytes = Bytes::from_string("0x123456789ABCDEF01234123456789abcdef01234").unwrap();
        assert_eq!(
            format!("{:?}", bytes),
            "Bytes([18, 52, 86, 120, 154, 188, 222, 240, 18, 52, 18, 52, 86, 120, 154, 188, 222, 240, 18, 52])"
        );

        let mut result = Bytes::from_string("0x123456789ABCDEF01234123456789abcdef");
        assert!(result.is_err());
        result = Bytes::from_string("0x123456789ABCDEF01234123456789abcdef0123412345");
        assert!(result.is_err());
        result = Bytes::from_string("0x123456789ABCDEF01234123456789abcdef01234123k");
        assert!(result.is_err());
    }

    #[test]
    fn equality() {
        let bytes_one = Bytes::from_string("0000000000000000000000000000000000000012").unwrap();
        let bytes_two = Bytes::from_string("0000000000000000000000000000000000000012").unwrap();
        let bytes_three = Bytes::from_string("0000000000000000000000000000000000000034").unwrap();
        let bytes_four = Bytes::from_string("00000000000000000000000000000000000012").unwrap();

        assert!(bytes_one == bytes_two);
        assert!(bytes_one != bytes_three);
        assert!(bytes_one != bytes_four);
    }
}
