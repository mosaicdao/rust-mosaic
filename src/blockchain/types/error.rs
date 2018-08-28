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

//! This module implements the Error struct and its methods.

use std;
use std::fmt;

/// An Error represents any error that appears during the interaction with the blockchain.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    explanation: String,
}

impl Error {
    /// A new error must have an ErrorKind and an explanation.
    pub fn new(kind: ErrorKind, explanation: String) -> Self {
        Error { kind, explanation }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    InvalidAddress,
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "A blockchain error occured."
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: is `unwrap()`ing the `write!`s here sensible?!
        match self.kind {
            ErrorKind::InvalidAddress => write!(f, "Not a valid address!").unwrap(),
        };

        write!(f, " Explanation: {}", self.explanation).unwrap();

        Ok(())
    }
}