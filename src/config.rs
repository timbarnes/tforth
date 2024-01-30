// system configuration and command line processing

use crate::engine::ForthInterpreter;
use crate::messages::DebugLevel;

use ::clap::{arg, Command};

const VERSION: &str = "0.1.24.1.30";
const WELCOME_MESSAGE: &str = "Welcome to tForth.";
const EXIT_MESSAGE: &str = "Finished";
const DEFAULT_CORE: &str = "forth/core.fs";

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
            core_file: DEFAULT_CORE.to_owned(),
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
            .arg(arg!(--library <VALUE>).required(false))
            .arg(arg!(--file <VALUE>).required(false))
            .arg(arg!(--nocore).required(false))
            .get_matches();

        let debuglevel = arguments.get_one::<String>("debuglevel");
        match debuglevel {
            Some(debuglevel) => match debuglevel.as_str() {
                "debug" => self.debug_level = DebugLevel::Debug,
                "info" => self.debug_level = DebugLevel::Info,
                "warning" => self.debug_level = DebugLevel::Warning,
                _ => self.debug_level = DebugLevel::Error,
            },
            None => {}
        }

        let library = arguments.get_one::<String>("library");
        match library {
            Some(lib) => self.core_file = lib.to_string(),
            None => {}
        }

        let nocore = arguments.get_one::<bool>("nocore");
        match nocore {
            Some(_nocore) => {
                println!("nocore: {}", _nocore);
                self.no_core = true;
            }
            None => self.no_core = false,
        }

        let file = arguments.get_one::<String>("file");
        match file {
            Some(file) => self.loaded_file = file.clone(),
            None => {}
        }
        return self;
    }

    pub fn run_forth(&mut self) {
        // create and run the interpreter
        // return when finished

        let mut forth = ForthInterpreter::new("Ok ", ">  ");

        forth.msg.set_level(self.debug_level.clone());

        if !self.no_core {
            if forth.load_file(self.core_file.as_str()) {
                self.loaded_core = true;
            } else {
                forth
                    .msg
                    .error("MAIN", "Unable to load core dictionary", &self.core_file);
            }
        }
        if self.loaded_file != "" {
            if !forth.load_file(self.loaded_file.as_str()) {
                forth
                    .msg
                    .error("MAIN", "Unable to load userfile", &self.loaded_file);
            }
        }
        /*
           // Define some Forth words
           interpreter.defined_words.insert(
               String::from("double"),
               vec![ForthToken::Number(2), ForthToken::Operator("*".to_string())],
           );
        */
        println!("{WELCOME_MESSAGE} Version {VERSION}");

        // Enter the interactive loop to read and process input
        loop {
            if forth.should_exit() {
                println!("{EXIT_MESSAGE}");
                break;
            }

            // Process one word (in immediate mode), or one definition (compile mode).
            if forth.process_token() {
                forth.msg.info("main", "   Stack", &forth.stack);
                forth.msg.debug("main", "   Words", &forth.defined_words);
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