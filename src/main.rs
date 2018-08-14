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

#[macro_use]
extern crate log;
extern crate mosaic;
extern crate simple_logger;

use log::Level;
use mosaic::config::Config;
use std::env;
use std::process;

const ENV_LOG_LEVEL: &str = "MOSAIC_LOG_LEVEL";
const DEFAULT_LOG_LEVEL: Level = Level::Info;

/// Reads the configuration and runs the node with it.
fn main() {
    let log_level = read_log_level();
    simple_logger::init_with_level(log_level).unwrap();

    let config = Config::new();

    if let Err(e) = mosaic::run(&config) {
        error!("Application error: {}", e);
        process::exit(1);
    }
}

/// Reads the log level from the environment. If it is not set it falls back to
/// the default log level.
/// It panics if a log level should be set that is not known.
fn read_log_level() -> Level {
    let log_level = env::var(ENV_LOG_LEVEL);
    match log_level {
        Ok(level) => match level.as_ref() {
            "TRACE" => Level::Trace,
            "DEBUG" => Level::Debug,
            "INFO" => Level::Info,
            "WARN" => Level::Warn,
            _ => panic!(
                "Unknown log level set. Allowed are: TRACE, DEBUG, INFO, WARN. Found: {}",
                level
            ),
        },
        Err(_) => DEFAULT_LOG_LEVEL,
    }
}
