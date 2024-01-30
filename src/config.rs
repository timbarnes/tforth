// system configuration and command line processing

use std::usize;

use crate::engine::ForthInterpreter;
use crate::messages::DebugLevel;
use crate::tokenizer::is_integer;

const VERSION: &str = "0.1.24.1.30";
const HELP_STRING: &str = "tforth 
  -h, --help                 - print this information
  -v, --version              - prints program version
  -d, --debuglevel n         - set debug level (0-4)
  -n, --nolibrary            - don't load the standard core library
  -l, --library <filename>   - load <filename> instead of the standard library
  -f, --file <filename>      - additional file of forth code to load";
const WELCOME_MESSAGE: &str = "Welcome to tForth.";
const EXIT_MESSAGE: &str = "Finished";

pub struct Config {
    version: String,
    args: Vec<String>,
    debug_level: DebugLevel,
    loaded_file: String,
    loaded_core: bool,
    core_file: String,
    help_string: String,
    pub run: bool,
}

impl Config {
    pub fn new(args: &Vec<String>) -> Config {
        Config {
            version: VERSION.to_owned(),
            args: args.clone(),
            debug_level: DebugLevel::Errors,
            loaded_file: "".to_owned(),
            loaded_core: false,
            core_file: "forth/core.fs".to_owned(),
            help_string: HELP_STRING.to_owned(),
            run: true,
        }
    }

    pub fn process_args(&mut self) -> &Config {
        // process arguments
        let count = self.args.len();
        if count == 1 {
            return self; // no arguments to process
        }
        for (i, arg) in self.args[1..].iter().enumerate() {
            match arg.as_str() {
                "-h" | "--help" => {
                    println!("{}", self.help_string);
                    self.run = false;
                }
                "-v" | "--version" => {
                    println!("{}", self.version);
                    self.run = false;
                }
                /*  "-d" | "--debuglevel" => {
                    let debuglevels = [
                        DebugLevel::Errors,
                        DebugLevel::Warnings,
                        DebugLevel::Info,
                        DebugLevel::Debug,
                    ];
                    let lev = &self.args[i + 1];
                    if is_integer(&lev) {
                        if i < count - 1 { // we have another argument
                            let lev = lev.parse::<usize>();
                            self.debug_level = debuglevels[lev.unwrap()];
                        }
                    } else {
                        println!("{}", self.help_string);
                        self.run == false;
                    }
                } */
                _ => {
                    println!("{}", self.help_string);
                    self.run = false;
                }
            }
        }
        return self;
    }

    pub fn run_forth(&mut self) {
        // create and run the interpreter
        // return when finished

        let mut forth = ForthInterpreter::new("Ok ", ">  ");

        forth.msg_handler.set_level(self.debug_level.clone());
        if !forth.load_file(self.core_file.as_str()) {
            forth
                .msg_handler
                .error("MAIN", "Unable to load core dictionary", &self.core_file);
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
                forth.msg_handler.info("main", "   Stack", &forth.stack);
                forth
                    .msg_handler
                    .debug("main", "   Words", &forth.defined_words);
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
