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

//! Basic types are hashes and numbers.
//!
//! Most big numbers are represented as arrays of primitive types. For example, `U128` is
//! represented as [u64; 2]. In that case, the `0` index of the array is the lower numbers and the
//! `1` index is the higher numbers. This means a decimal `1` would be represented as `[1, 0]`.

use std::fmt::{self, Formatter, LowerHex};

/// H256 is a 256-bit hash.
#[derive(Debug, Copy, PartialEq)]
pub struct H256(pub [u8; 32]);

impl H256 {
    /// Returns the underlying `u8` array.
    pub fn bytes(&self) -> [u8; 32] {
        self.0
    }
}

impl Clone for H256 {
    fn clone(&self) -> H256 {
        *self
    }
}

impl From<[u8; 32]> for H256 {
    /// Converts a `u8` array of 32 items into an `H256`.
    fn from(bytes: [u8; 32]) -> Self {
        Self { 0: bytes }
    }
}

impl LowerHex for H256 {
    /// Writes the bytes as hex with leading zeros to the given Formatter.
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        for byte in self.bytes().iter() {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

/// U128 is a 128-bit unsigned integer.
#[derive(Debug, Copy, PartialEq)]
pub struct U128(pub [u64; 2]);

impl U128 {
    /// Returns the underlying `u64` array.
    pub fn bytes(&self) -> [u64; 2] {
        self.0
    }
}

impl Clone for U128 {
    fn clone(&self) -> U128 {
        *self
    }
}

impl From<[u64; 2]> for U128 {
    /// Converts a `u64` array of 2 items into a `U128`.
    fn from(bytes: [u64; 2]) -> Self {
        Self { 0: bytes }
    }
}

impl From<U128> for u64 {
    /// Tries to convert a U128 to a u64. Panics on overflow.
    fn from(u128: U128) -> u64 {
        if u128.0[1] != 0 {
            panic!("Overflow when converting U128 to u64.");
        }

        u128.0[0]
    }
}

impl LowerHex for U128 {
    /// Writes the bytes as hex with leading zeros to the given Formatter.
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        for byte in self.bytes().iter() {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

/// U256 is a 256-bit unsigned integer.
#[derive(Debug, Copy, PartialEq)]
pub struct U256(pub [u64; 4]);

impl U256 {
    /// Returns the underlying `u64` array.
    pub fn bytes(&self) -> [u64; 4] {
        self.0
    }
}

impl Clone for U256 {
    fn clone(&self) -> U256 {
        *self
    }
}

impl From<[u64; 4]> for U256 {
    /// Converts a `u64` array of 4 items into a `U256`.
    fn from(bytes: [u64; 4]) -> Self {
        Self { 0: bytes }
    }
}

impl LowerHex for U256 {
    /// Writes the bytes as hex with leading zeros to the given Formatter.
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        for byte in self.bytes().iter() {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}
