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

//! This module implements errors that are specific to mosaic.

use std::fmt;

/// An Error represents any error that happens during the execution of rust-mosaic.
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

/// The kinds of errors that can appear.
#[derive(Debug)]
pub enum ErrorKind {
    AbiError,
    InvalidBlock,
    NodeError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::AbiError => write!(f, "ABI error!").unwrap(),
            ErrorKind::InvalidBlock => write!(f, "Not a valid block!").unwrap(),
            ErrorKind::NodeError => write!(f, "Error on blockchain node!").unwrap(),
        };

        write!(f, " Explanation: {}", self.explanation).unwrap();

        Ok(())
    }
}
