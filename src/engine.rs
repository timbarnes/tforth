//The Tforth interpreter struct and implementation

use std::collections::HashMap;

use crate::messages::{DebugLevel, Msg};
use crate::utility::is_number;

#[derive(Debug, Clone)]
pub struct ForthInterpreter {
    pub stack: Vec<i32>, // the numeric stack, currently integers
    pub defined_words: HashMap<String, Vec<ForthToken>>, // the dictionary: keys (words) and their definitions
    pub exit_flag: bool,                                 // set when the "bye" word is executed.
    pub msg_handler: Msg,
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
            exit_flag: false,
            msg_handler: Msg::new(),
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

    fn execute_operator(&mut self, operator: &str) {
        self.msg_handler
            .info("execute_operator", format!("operator is {operator}"));
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
                        .warning("DUP", "Error - DUP: Stack is empty.".to_owned());
                }
            }
            "drop" => {
                if self.stack.len() > 0 {
                    self.stack.pop();
                } else {
                    self.msg_handler
                        .warning("DROP", "Stack is empty.".to_owned());
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
                        .warning("DUP", "Too few elements on stack.".to_owned());
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

    fn execute_tokens(&mut self, tokens: &[ForthToken]) {
        self.msg_handler
            .info("execute_tokens", format!("tokens: {:?}", tokens));
        for token in tokens {
            self.msg_handler.info(
                "execute_tokens",
                format!("...token is: {:?}, stack is {:?}", token, self.stack),
            );
            match token {
                ForthToken::Number(num) => self.stack.push(*num),
                ForthToken::Operator(op) => self.execute_operator(&op),
            }
        }
    }

    fn execute_word(&mut self, word: &str) {
        if let Some(tokens) = self.defined_words.clone().get(word) {
            self.msg_handler
                .info("execute_word", format!("tokens: {:?}", tokens));
            self.execute_tokens(tokens);
        } else {
            println!("Error - Unknown word: {}", word);
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
        self.msg_handler.info(
            "parse_word_definition",
            format!("token vector: {:?}", input),
        );
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
