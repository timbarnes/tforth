// Read tokens from a file or stdin, one line at a time.
// Return one space-delimited token at a time.
// Cache the remainder of the line.

use std::collections::VecDeque;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use crate::messages::{DebugLevel, Msg};

#[derive(Debug)]
enum TokenSource {
    Stdin,
    Stream(BufReader<File>),
}

pub struct Tokenizer {
    source: TokenSource,
    line: String,             // the text of the current line
    tokens: VecDeque<String>, // a vector of tokens, successively popped off until empty and a new line is read.
    success: bool, // Set to true if the Tokenizer was properly created. Fails on file errors
    msg: Msg,
}

impl fmt::Debug for Tokenizer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Tokenizer")
            .field(&self.source)
            .field(&self.line)
            .field(&self.tokens)
            .field(&self.success)
            .finish()
    }
}

impl Tokenizer {
    pub fn new(file_path: Option<&str>) -> Tokenizer {
        // Initialize a tokenizer.
        let mut message_handler = Msg::new();
        message_handler.set_level(DebugLevel::No);
        match file_path {
            None => {
                return Tokenizer {
                    source: TokenSource::Stdin,
                    line: String::new(),
                    tokens: VecDeque::new(),
                    success: true,
                    msg: message_handler,
                }
            }
            Some(filepath) => {
                let file = File::open(filepath);
                match file {
                    Ok(file) => {
                        return Tokenizer {
                            source: TokenSource::Stream(BufReader::new(file)),
                            line: String::new(),
                            tokens: VecDeque::new(),
                            success: true,
                            msg: message_handler,
                        }
                    }
                    Err(_) => {
                        let tkn = Tokenizer {
                            source: TokenSource::Stdin,
                            line: String::new(),
                            tokens: VecDeque::new(),
                            success: false,
                            msg: message_handler,
                        };
                        tkn.msg
                            .error("Tokenizer::new", "File not able to be opened", file_path);
                        return tkn;
                    }
                }
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        self.success
    }

    fn get_line(&mut self) -> bool {
        // Read a line, storing it if there is one
        // In interactive (stdin) mode, blocks until the user provides a line.
        // Returns true if a line was read, false if not.
        match self.source {
            TokenSource::Stdin => {
                // Read from stdin
                self.line.clear();
                if let Err(_) = io::stdin().read_line(&mut self.line) {
                    return false;
                } else {
                    self.msg.info("get_line", "Got some values", &self.line);
                    self.tokenize();
                    self.msg.info("get_line", "tokens are", &self.tokens);
                    return true;
                }
            }
            TokenSource::Stream(ref mut file) => {
                // Read from a file. TokenSource is a BufReader
                self.msg.info("get_line", "Reading from file", "");
                if let Err(_) = &file.read_line(&mut self.line) {
                    return false;
                } else {
                    self.msg.info("self.line", "Text was", &self.line);
                    return true;
                }
            }
        }
    }

    fn tokenize(&mut self) -> bool {
        // Tokenize the current line, returning true if successful
        // Only fails if the line is empty
        //
        if self.line.is_empty() {
            return false;
        } else {
            let mut inside_quotes = false;
            let mut current_word = String::new();
            for c in self.line.chars() {
                match c {
                    ' ' | '\n' if !inside_quotes => {
                        if !current_word.is_empty() {
                            self.tokens.push_back(current_word.clone());
                            current_word.clear();
                        }
                    }
                    '\"' => inside_quotes = !inside_quotes,
                    _ => current_word.push(c),
                }
            }
            if !current_word.is_empty() {
                self.tokens.push_back(current_word);
            }
        }
        self.tokens.len() > 0
    }

    pub fn get_token(&mut self) -> Option<String> {
        // Returns the next token from the stream, if there is one, otherwise None
        if self.tokens.is_empty() {
            self.msg
                .info("get_token", "no more tokens; need a new line", "");
            if self.get_line() {
                let t = self.tokens.pop_front();
                match t {
                    Some(t) => {
                        return Some(t);
                    }
                    _ => {
                        return None;
                    }
                }
            } else {
                self.msg.error("get_token", "unable to get new line", "");
                return None;
            }
        } else {
            return self.tokens.pop_front();
        }
    }
}
