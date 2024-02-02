// Tforth main program
// Version 0.1

mod config;
mod doc;
mod engine;
mod messages;
mod reader;
mod tokenizer;

use config::Config;

fn main() {
    let mut config = Config::new();
    config.process_args();

    if config.run {
        config.run_forth();
    } else {
        config.exit()
    }
}
