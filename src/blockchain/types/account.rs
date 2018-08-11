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

//! This module implements the Account struct and its methods.

use std::cmp::PartialEq;
use std::fmt::{Debug, Error, Formatter, LowerHex};

/// An Account is represented by a 20-bytes address.
pub struct Account([u8; 20]);

impl Account {
    /// Creates an account with the address given as 20 bytes.
    pub fn new(bytes: [u8; 20]) -> Self {
        Account(bytes)
    }

    /// Returns the bytes representation of this account.
    pub fn bytes(&self) -> [u8; 20] {
        self.0
    }
}

impl PartialEq for Account {
    /// Two accounts are equal if their byte representations are equal.
    fn eq(&self, other: &Account) -> bool {
        self.bytes() == other.bytes()
    }
}

impl LowerHex for Account {
    /// Writes the bytes as hex with leading zeros to the given Formatter.
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl Debug for Account {
    /// Debug format is lower hex.
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{:x}", self)?;

        Ok(())
    }
}

// Converting other types to and from Accounts
pub trait AsAccount {
    fn as_account(&self) -> Account;
}

pub trait FromAccount {
    fn from_account(account: Account) -> Self;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn account_to_lower_hex() {
        let mut bytes = [0u8; 20];
        let account = Account::new(bytes);
        assert_eq!(
            format!("{:x}", account),
            "0000000000000000000000000000000000000000"
        );

        bytes[0] = 1u8;
        let account = Account::new(bytes);
        assert_eq!(
            format!("{:x}", account),
            "0100000000000000000000000000000000000000"
        );

        bytes[19] = 18u8;
        let account = Account::new(bytes);
        assert_eq!(
            format!("{:x}", account),
            "0100000000000000000000000000000000000012"
        );
    }
}
