// Copyright 2018 OpenST Ltd.
// Copyright 2018 OpenST Ltd.
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

use web3::futures::Future;
use web3::transports::{EventLoopHandle, Http};
use web3::types::H160;
use web3::Web3;

use super::types::account::{Account, AsAccount, FromAccount};
use super::Blockchain;

/// This struct stores a connection to an Ethereum node.
pub struct Ethereum {
    web3: Web3<Http>,
    _eloop: EventLoopHandle,
}

impl Ethereum {
    /// Creates a new instance of Ethereum pointing to the given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of an ethereum node.
    pub fn new(address: String) -> Blockchain {
        let (eloop, http) = Http::new(address.as_str()).unwrap();
        let web3 = Web3::new(http);

        Blockchain::Eth(Ethereum {
            web3,
            _eloop: eloop,
        })
    }

    /// Uses web3 to retrieve the accounts.
    /// Converts them to blockchain accounts and returns all accounts in a
    /// vector.
    pub fn get_accounts(&self) -> Vec<Account> {
        let accounts = self.web3.eth().accounts().wait().unwrap();
        let mut v = Vec::new();

        for h160 in accounts {
            v.push(h160.as_account())
        }

        v
    }
}

impl AsAccount for H160 {
    /// Converts an H160 type to an Account.
    /// The account's bytes will be a copy of H160.
    fn as_account(&self) -> Account {
        let mut bytes: [u8; 20] = [b'0'; 20];
        self.copy_to(&mut bytes);

        Account::new(bytes)
    }
}

impl FromAccount for H160 {
    /// Creates an H160 type from an account.
    /// H160 will equal the account's bytes.
    fn from_account(account: Account) -> Self {
        H160::from(account.bytes())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_h160_to_account() {
        let mut bytes = [0u8; 20];
        assert_eq!(
            "0000000000000000000000000000000000000000"
                .parse::<H160>()
                .unwrap()
                .as_account(),
            Account::new(bytes)
        );

        bytes[19] = 10u8;
        assert_eq!(
            "000000000000000000000000000000000000000a"
                .parse::<H160>()
                .unwrap()
                .as_account(),
            Account::new(bytes)
        );

        bytes[0] = 1u8;
        assert_eq!(
            "010000000000000000000000000000000000000a"
                .parse::<H160>()
                .unwrap()
                .as_account(),
            Account::new(bytes)
        );
    }

    #[test]
    fn test_h160_from_account() {
        let mut bytes = [0u8; 20];
        assert_eq!(
            format!("{:#?}", H160::from_account(Account::new(bytes))),
            "0x0000000000000000000000000000000000000000"
        );

        bytes[19] = 10u8;
        assert_eq!(
            format!("{:#?}", H160::from_account(Account::new(bytes))),
            "0x000000000000000000000000000000000000000a"
        );

        bytes[0] = 1u8;
        assert_eq!(
            format!("{:#?}", H160::from_account(Account::new(bytes))),
            "0x010000000000000000000000000000000000000a"
        );
    }
}
