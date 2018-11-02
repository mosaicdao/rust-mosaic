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

use super::blockchain::Blockchain;
use futures::prelude::*;

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
pub fn run(origin: &Blockchain, auxiliary: &Blockchain, event_loop: &tokio_core::reactor::Handle) {
    let origin_stream = origin.stream_blocks();
    let auxiliary_stream = auxiliary.stream_blocks();

    // `info!`s are just used as an example. The actual logic of how to handle each block will be
    // done here. Should spawn new futures to not block if longer computation.
    let origin_worker = origin_stream.map_err(|_| ()).for_each(|block| {
        info!("Origin Block:    {}", block);
        Ok(())
    });
    let auxiliary_worker = auxiliary_stream.map_err(|_| ()).for_each(|block| {
        info!("Auxiliary Block: {}", block);
        Ok(())
    });

    event_loop.spawn(origin_worker);
    event_loop.spawn(auxiliary_worker);

    // Below here is only example code to see how it works:
    let signature = origin.sign(vec![1, 2, 3, 4].into());
    event_loop.spawn(
        signature
            .map_err(|error| error!("Could not sign in observer: {}", error))
            .and_then(|signature| {
                info!("Signature: {:x}", signature);
                Ok(())
            }),
    );

    let accounts = origin
        .get_accounts()
        .map_err(|error| error!("Could not get accounts: {}", error))
        .and_then(|accounts| {
            info!("Received accounts: {:?}", accounts);
            Ok(())
        });
    event_loop.spawn(accounts);
}
