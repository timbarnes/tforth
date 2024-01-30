// Tforth main program
// Version 0.1

mod config;
mod engine;
mod messages;
mod reader;
mod tokenizer;

use config::Config;
use std::env;

fn main() {
    let args = env::args().collect();
    let mut config = Config::new(&args);
    config.process_args();

    if config.run {
        config.run_forth();
    } else {
        config.exit()
    }
}
