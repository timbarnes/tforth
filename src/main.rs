use std::io::{self, Write};

mod engine;
mod messages;
mod utility;

use engine::{ForthInterpreter, ForthToken};
use messages::DebugLevel;

fn main() {
    let mut interpreter = ForthInterpreter::new();

    interpreter.msg_handler.set_level(DebugLevel::No);

    /* // Define some Forth words
       interpreter.defined_words.insert(
           String::from("double"),
           vec![ForthToken::Number(2), ForthToken::Operator("*".to_string())],
       );
    */
    println!("Welcome to Tforth, my first real Rust program!");
    println!("Message level is {:?}", interpreter.msg_handler.get_level());

    // Enter the interactive loop to read and process input
    loop {
        if interpreter.should_exit() {
            println!("Thank you for using Tforth!");
            break;
        }
        print!("Ok ");
        io::stdout().flush().unwrap();

        let mut input = String::new();

        // Read a line from stdin
        if let Err(_) = io::stdin().read_line(&mut input) {
            interpreter
                .msg_handler
                .warning("REPL", "Error - Reading input", "Stdin");
            break;
        }

        if let Some((name, definition)) = interpreter.parse_word_definition(&input) {
            interpreter.define_word(&name, &definition);
            interpreter.msg_handler.info("main", "Defined word", name);
        } else {
            // Split the input into tokens
            let tokens: Vec<ForthToken> = input
                .split_whitespace()
                .map(|token| {
                    if utility::is_number(token) {
                        ForthToken::Number(token.parse().unwrap())
                    } else {
                        ForthToken::Operator(token.to_string())
                    }
                })
                .collect();

            // Execute the Forth program
            interpreter.execute(&tokens);

            // Print the current stack
        }
        interpreter
            .msg_handler
            .info("main", "   Stack", &interpreter.stack);
        interpreter
            .msg_handler
            .info("main", "   Words", &interpreter.defined_words);
    }
}
