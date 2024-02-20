// system configuration and command line processing

use crate::engine::TF;
use crate::messages::DebugLevel;

use ::clap::{arg, Command};

const VERSION: &str = "alpha.24.2.20";
const WELCOME_MESSAGE: &str = "Welcome to tForth.";
const EXIT_MESSAGE: &str = "Finished";
const DEFAULT_CORE: [&str; 2] = ["~/.tforth/corelib.fs", "src/corelib.fs"];

pub struct Config {
    debug_level: DebugLevel,
    loaded_file: String,
    loaded_core: bool,
    core_file: String,
    no_core: bool,
    pub run: bool,
}

impl Config {
    pub fn new() -> Config {
        Config {
            debug_level: DebugLevel::Error,
            loaded_file: "".to_owned(),
            loaded_core: false,
            core_file: DEFAULT_CORE[0].to_owned(),
            no_core: false,
            run: true,
        }
    }

    pub fn process_args(&mut self) -> &Config {
        // process arguments
        // let msg = Msg::new(); // Create a message handler for argument errors

        let arguments = Command::new("tForth")
            .version(VERSION)
            .author("Tim Barnes")
            .about("A simple Forth interpreter")
            .arg(
                arg!(--debuglevel <VALUE>)
                    .required(false)
                    .value_parser(["error", "warning", "info", "debug"]),
            )
            .arg(arg!(-l --library <VALUE>).required(false))
            .arg(arg!(-f --file <VALUE>).required(false))
            .arg(arg!(-n - -nocore).required(false))
            .get_matches();

        let debuglevel = arguments.get_one::<String>("debuglevel");
        if let Some(debuglevel) = debuglevel {
            match debuglevel.as_str() {
                "debug" => self.debug_level = DebugLevel::Debug,
                "info" => self.debug_level = DebugLevel::Info,
                "warning" => self.debug_level = DebugLevel::Warning,
                _ => self.debug_level = DebugLevel::Warning,
            }
        }

        let library = arguments.get_one::<String>("library");
        if let Some(lib) = library {
            self.core_file = lib.to_string();
        }

        let nocore = arguments.get_one::<bool>("nocore");
        if let Some(nc) = nocore {
            self.no_core = *nc;
        }

        let file = arguments.get_one::<String>("file");
        if let Some(file) = file {
            self.loaded_file = file.clone();
        }
        self
    }

    pub fn run_forth(&mut self) {
        // create and run the interpreter
        // return when finished

        let mut forth = TF::new("Ok ", ">  ");
        forth.cold_start();

        forth.msg.set_level(self.debug_level.clone());

        if !self.no_core {
            for path in DEFAULT_CORE {
                if forth.load_file(&path.to_owned()) {
                    self.loaded_core = true;
                    forth
                        .msg
                        .info("MAIN", "Loaded core dictionary", Some(&self.core_file));
                    break;
                }
            }
            if !self.loaded_core {
                forth.msg.error(
                    "MAIN",
                    "Unable to load core dictionary",
                    Some(&self.core_file),
                );
            }
        }
        if self.loaded_file != "" {
            if !forth.load_file(&self.loaded_file) {
                forth
                    .msg
                    .error("MAIN", "Unable to load userfile", Some(&self.loaded_file));
            }
        }

        forth.set_abort_flag(false); // abort flag may have been set by load_file, but is no longer needed.

        println!("{WELCOME_MESSAGE} Version {VERSION}");

        // Enter the interactive loop to read and process input
        loop {
            if forth.should_exit() {
                println!("{EXIT_MESSAGE}");
                break;
            }

            // Process one word (in immediate mode), or one definition (compile mode).
            if forth.process_token() {
                forth.msg.info("main", "   Stack", Some(&forth.stack));
                // forth.msg.debug("main", "   Words", &forth.defined_words);
            } else {
                // Exit if EOF.
                println!("End of File. Thank you for using tForth!");
                break;
            }
        }
    }

    pub fn exit(&self) {
        println!("{EXIT_MESSAGE}");
    }
}
