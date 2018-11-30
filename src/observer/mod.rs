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
//! It has a `run` function that creates connections to origin and auxiliary and then takes defined
//! actions for each new block that it observes on the connected chains.

use super::error::Error;
use super::ethereum::{Block, Ethereum};
use super::event;
use super::Config;
use futures::prelude::*;
use std::sync::Arc;

/// Runs a mosaic observer. The observer observes blocks from origin and auxiliary. When a new block
/// is observed, the observer hands new  tasks to the reactor, based on the block origin and
/// content.
///
/// Observations are handled as streams that are added to the given event loop.
///
/// # Arguments
///
/// * `origin` - A blockchain object that points to origin.
/// * `auxiliary` - A blockchain object that points to auxiliary.
/// * `event_loop` - The reactor's event loop to handle the tasks spawned by this observer.
/// * `config` - The configuration object of mosaic.
pub fn run(
    origin: &Ethereum,
    auxiliary: &Ethereum,
    event_loop: &tokio_core::reactor::Handle,
    config: &Config,
) {
    let origin_events = event::origin_event_registry(config);
    let auxiliary_events = event::auxiliary_event_registry(config);

    let origin_stream = origin.stream_blocks(Arc::new(origin_events));
    let auxiliary_stream = auxiliary.stream_blocks(Arc::new(auxiliary_events));

    let origin_worker = worker(origin_stream, origin_block_function);
    let auxiliary_worker = worker(auxiliary_stream, auxiliary_block_function);

    event_loop.spawn(origin_worker);
    event_loop.spawn(auxiliary_worker);
}

/// A worker takes a block stream and a function to apply to each block. The function takes the
/// block as an argument and returns a result. If it returns an error the error will be logged.
///
/// # Arguments
///
/// * `block_stream` - A stream of block items.
/// * `block_function` - A function that will be called with every block as an argument.
fn worker<F>(
    block_stream: impl Stream<Item = Block, Error = Error>,
    block_function: F,
) -> impl Future<Item = (), Error = ()>
where
    F: Fn(&Block) -> Result<(), Error>,
{
    // Using `then` to catch errors. If the errors weren't caught, the stream would terminate after
    // an error. However, we want to continue polling the node for new blocks, even if there was an
    // error with a particular block. In the `for_each` block we need to then check for an existing
    // block as we caught all blocks and errors and mapped both to `Option`al blocks (`None` in the
    // error case).
    block_stream
        .then(|item| match item {
            Ok(block) => Ok(Some(block)),
            Err(error) => {
                error!("Error when streaming blocks: {}", error);
                Ok(None)
            }
        }).for_each(move |block| {
            let block = match block {
                Some(block) => block,
                None => return Ok(()),
            };

            // Here we actually call the block function that does the actual work. The rest around
            // it is more or less boilerplate.
            if let Err(error) = block_function(&block) {
                error!("There was an error when processing a block: {}", error);
            }

            Ok(())
        })
}

/// origin_block_function implements the actions that should be taken for each block that we observe
/// on origin.
fn origin_block_function(block: &Block) -> Result<(), Error> {
    // `info!`s are just used as an example. The actual logic of how to handle each block will be
    // done here. Should spawn new futures to not block if longer computation.
    info!("Origin Block:     {}", block);
    info!("Origin Events:    {:?}", block.events);

    Ok(())
}

/// origin_block_function implements the actions that should be taken for each block that we observe
/// on auxiliary.
fn auxiliary_block_function(block: &Block) -> Result<(), Error> {
    // `info!`s are just used as an example. The actual logic of how to handle each block will be
    // done here. Should spawn new futures to not block if longer computation.
    info!("Auxiliary Block:     {}", block);
    info!("Auxiliary Events:    {:?}", block.events);

    Ok(())
}
