//The Tforth interpreter struct and implementation

use std::collections::HashMap;

use crate::messages::{DebugLevel, Msg};
use crate::tokenizer::Tokenizer;
use crate::utility::is_number;

#[derive(Debug)]
pub struct ForthInterpreter {
    pub stack: Vec<i32>, // the numeric stack, currently integers
    pub defined_words: HashMap<String, Vec<ForthToken>>, // the dictionary: keys (words) and their definitions
    compile_mode: bool,                                  // true if compiling a word
    exit_flag: bool,                                     // set when the "bye" word is executed.
    pub msg_handler: Msg,
    tokenizer: Tokenizer,
    new_word_name: String,
    new_word_definition: Vec<ForthToken>,
}

#[derive(Debug, Clone)]
pub enum ForthToken {
    Number(i32),      // the token is an integer, stored here
    Operator(String), // the token is an operator, hardcoded
}

impl ForthInterpreter {
    // ForthInterpreter struct implementations
    pub fn new() -> ForthInterpreter {
        ForthInterpreter {
            stack: Vec::new(),
            defined_words: HashMap::new(),
            compile_mode: false,
            exit_flag: false,
            msg_handler: Msg::new(),
            tokenizer: Tokenizer::new(None),
            new_word_name: String::new(),
            new_word_definition: Vec::new(),
        }
    }

    fn set_exit_flag(&mut self) {
        // Method executed by "bye"
        self.exit_flag = true;
    }

    pub fn should_exit(&self) -> bool {
        // Method to determine if we should exit
        self.exit_flag
    }

    fn get_compile_mode(&self) -> bool {
        self.compile_mode
    }

    fn set_compile_mode(&mut self, state: bool) {
        self.compile_mode = state;
    }

    fn execute_operator(&mut self, operator: &str) {
        self.msg_handler
            .info("execute_operator", "operator is", operator);
        match operator {
            "+" => {
                if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                    self.stack.push(a + b);
                }
            }
            "-" => {
                if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                    self.stack.push(b - a);
                }
            }
            "*" => {
                if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                    self.stack.push(a * b);
                }
            }
            "/" => {
                if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                    self.stack.push(b / a);
                }
            }
            "." => {
                if let Some(a) = self.stack.pop() {
                    println!("{a}");
                }
            }
            "true" => {
                self.stack.push(0);
            }
            "false" => {
                self.stack.push(-1);
            }
            "=" => {
                if let Some(a) = self.stack.pop() {
                    if let Some(b) = self.stack.pop() {
                        if a == b {
                            self.stack.push(0)
                        } else {
                            self.stack.push(-1);
                        }
                    }
                }
            }
            "0=" => {
                if let Some(a) = self.stack.pop() {
                    if a == 0 {
                        self.stack.push(0)
                    } else {
                        self.stack.push(-1);
                    }
                }
            }
            "0<" => {
                if let Some(a) = self.stack.pop() {
                    if a < 0 {
                        self.stack.push(0)
                    } else {
                        self.stack.push(-1);
                    }
                }
            }
            ".s" => {
                println!("{:?}", self.stack);
            }
            "clear" => {
                self.stack.clear();
            }
            "dup" => {
                if let Some(top) = self.stack.last() {
                    self.stack.push(*top);
                } else {
                    self.msg_handler
                        .warning("DUP", "Error - DUP: Stack is empty.", "");
                }
            }
            "drop" => {
                if self.stack.len() > 0 {
                    self.stack.pop();
                } else {
                    self.msg_handler.warning("DROP", "Stack is empty.", "");
                }
            }
            "swap" => {
                if self.stack.len() > 1 {
                    let a = self.stack[self.stack.len() - 1];
                    let b = self.stack[self.stack.len() - 2];
                    self.stack.pop();
                    self.stack.pop();
                    self.stack.push(a);
                    self.stack.push(b);
                } else {
                    self.msg_handler
                        .warning("DUP", "Too few elements on stack.", "");
                }
            }
            "words" => {
                for (key, _) in self.defined_words.iter() {
                    print!("{key} ");
                }
            }
            "wordsee" => {
                for (key, value) in self.defined_words.iter() {
                    print!(": {key} ");
                    for word in value {
                        match word {
                            ForthToken::Number(num) => print!("{num} "),
                            ForthToken::Operator(op) => print!("{op} "),
                        }
                    }
                    println!(";");
                }
            }
            "debuglevel" => match self.stack.pop() {
                Some(0) => self.msg_handler.set_level(DebugLevel::No),
                Some(1) => self.msg_handler.set_level(DebugLevel::Warning),
                _ => self.msg_handler.set_level(DebugLevel::Info),
            },
            "debuglevel?" => {
                println!("DebugLevel is {:?}", self.msg_handler.get_level());
            }
            "bye" => {
                self.set_exit_flag();
            }
            // Add more operators as needed
            _ => {
                self.execute_word(operator);
            }
        }
    }

    pub fn process_item(&mut self) -> bool {
        // Process one immediate word, or a definition
        let tok = self.tokenizer.get_token();
        match tok {
            Some(token) => {
                if token == ":" {
                    return self.process_definition();
                } else {
                    self.process_word(&token);
                    return true;
                }
            }
            None => false,
        }
    }

    fn process_word(&mut self, token: &String) {
        // Process a single word in immediate mode
        if is_number(token) {
            self.stack.push(token.parse().unwrap());
        } else {
            self.execute_operator(&token);
        }
    }

    fn process_definition(&mut self) -> bool {
        // Process the definition of a new word
        self.set_compile_mode(true);
        let name = self.tokenizer.get_token();
        match name {
            Some(name) => {
                self.new_word_name = name;
            }
            None => {
                self.msg_handler
                    .error("process_definition", "Name not found", "");
                return false;
            }
        }
        loop {
            // Loop over the definition
            let tok = self.tokenizer.get_token();
            match tok {
                Some(token) => {
                    if is_number(&token) {
                        self.new_word_definition
                            .push(ForthToken::Number(token.parse().unwrap()));
                    } else if token != ";" {
                        // ; is end of definition
                        self.new_word_definition
                            .push(ForthToken::Operator(token.to_string()));
                    } else {
                        break; // We found the end of the definition
                    }
                }
                None => {
                    return false;
                }
            }
        }
        self.defined_words
            .insert(self.new_word_name.clone(), self.new_word_definition.clone());
        self.set_compile_mode(false);
        return true;
    }

    fn execute_tokens(&mut self, tokens: &[ForthToken]) {
        self.msg_handler.info("execute_tokens", "tokens", tokens);
        for token in tokens {
            self.msg_handler
                .info("execute_tokens", "...token and stack", token);
            match token {
                ForthToken::Number(num) => self.stack.push(*num),
                ForthToken::Operator(op) => self.execute_operator(&op),
            }
        }
    }

    fn execute_word(&mut self, word: &str) {
        if let Some(tokens) = self.defined_words.clone().get(word) {
            self.msg_handler.info("execute_word", "tokens", tokens);
            self.execute_tokens(tokens);
        } else {
            self.msg_handler.error("execute_word", "Unknown word", word);
        }
    }

    pub fn execute(&mut self, tokens: &[ForthToken]) {
        self.execute_tokens(tokens);
    }

    pub fn define_word(&mut self, name: &str, definition: &[ForthToken]) {
        self.defined_words
            .insert(name.to_string(), definition.to_vec());
    }

    pub fn parse_word_definition(&self, input: &str) -> Option<(String, Vec<ForthToken>)> {
        // Compile a word
        self.msg_handler
            .info("parse_word_definition", "token vector", input);
        let mut iter = input.split_whitespace();
        if let Some(token) = iter.next() {
            if token == ":" {
                if let Some(name) = iter.next() {
                    let mut definition = Vec::new();
                    while let Some(token) = iter.next() {
                        if token == ";" {
                            return Some((name.to_string(), definition));
                        } else {
                            // Parse the token into ForthToken
                            if is_number(token) {
                                definition.push(ForthToken::Number(token.parse().unwrap()));
                            } else {
                                definition.push(ForthToken::Operator(token.to_string()));
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
