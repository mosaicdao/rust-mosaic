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

//! This module is about observing blockchains.

use futures::prelude::*;
use std::sync::Arc;

use ethereum::Ethereum;
use reactor::React;

/// This represents as observer of a block chain.
pub struct Observer {
    block_chain: Arc<Ethereum>,
    /// List of block reactors. These are notified when any new block is generated.
    reactors: Arc<Vec<Box<React>>>,
    event_loop: Box<tokio_core::reactor::Handle>,
}

impl Observer {
    /// Instantiate an observer.
    ///
    /// # Arguments
    ///
    /// * `origin` - A blockchain object that points to origin.
    /// * `auxiliary` - A blockchain object that points to auxiliary.
    /// * `event_loop` - The reactor's event loop to handle the tasks spawned by this observer.
    pub fn new(
        block_chain: Arc<Ethereum>,
        reactors: Arc<Vec<Box<React>>>,
        event_loop: Box<tokio_core::reactor::Handle>,
    ) -> Self {
        Observer {
            block_chain,
            reactors,
            event_loop,
        }
    }
    /// Runs a mosaic observer. The observer observes blocks from a block chain. When a new block
    /// is observed, the observer hands new  tasks to the reactor,
    ///
    /// Observations are handled as streams that are added to the given event loop.
    ///
    pub fn run(&self) {
        // Using `then` to catch errors. If the errors weren't caught, the stream would terminate after
        // an error. However, we want to continue polling the node for new blocks, even if there was an
        // error with a particular block. In the `for_each` block we need to then check for an existing
        // block as we caught all blocks and errors and mapped both to `Option`al blocks (`None` in the
        // error case).
        let worker = self
            .block_chain
            .stream_blocks()
            .then(|item| match item {
                Ok(block) => Ok(Some(block)),
                Err(error) => {
                    error!("Error when streaming from chain: {}", error);
                    Ok(None)
                }
            }).for_each({
                let reactors = Arc::clone(&self.reactors);
                move |block| {
                    let block = match block {
                        Some(block) => block,
                        None => return Ok(()),
                    };

                    reactors.iter().for_each(|reactor| reactor.react(&block));
                    Ok(())
                }
            });

        self.event_loop.spawn(worker);
    }
}
