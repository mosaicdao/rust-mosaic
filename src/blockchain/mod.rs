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

//! This module provides an API to interact with blockchains, e.g. Ethereum.

mod ethereum;

/// The Blockchain represents all shared functions of a blockchain.
pub trait Blockchain {
    /// Returns a vector of all accounts in hex format.
    fn get_accounts(&self) -> Vec<String>;
}

/// Creates a new ethereum blockchain.
pub fn new_ethereum(address: String) -> Box<Blockchain> {
    let ethereum = ethereum::Ethereum::new(address);
    Box::new(ethereum)
}

#[cfg(test)]
mod test {}
