// Tforth main program
// Version 0.1

mod engine;
mod messages;
mod reader;
mod tokenizer;
mod utility;

use engine::ForthInterpreter;
use messages::DebugLevel;

fn main() {
    let mut interpreter = ForthInterpreter::new("Ok ", ">  ");

    interpreter.msg_handler.set_level(DebugLevel::Info);
    if !interpreter.load_file("forth/core.fs") {
        interpreter
            .msg_handler
            .error("MAIN", "Unable to load core dictionary", "core.fs")
    }

    /* // Define some Forth words
       interpreter.defined_words.insert(
           String::from("double"),
           vec![F
           orthToken::Number(2), ForthToken::Operator("*".to_string())],
       );
    */
    println!("Welcome to tForth, my first real Rust program!");
    println!("Message level is {:?}", interpreter.msg_handler.get_level());

    // Enter the interactive loop to read and process input
    loop {
        if interpreter.should_exit() {
            println!("Thank you for using Tforth!");
            break;
        }

        // Process one word (in immediate mode), or one definition (compile mode).
        if interpreter.process_token() {
            interpreter
                .msg_handler
                .info("main", "   Stack", &interpreter.stack);
            interpreter
                .msg_handler
                .debug("main", "   Words", &interpreter.defined_words);
        } else {
            // Exit if EOF.
            println!("End of File. Thank you for using tForth!");
            break;
        }
    }
}
