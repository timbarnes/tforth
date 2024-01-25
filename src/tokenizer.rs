// Read tokens from a file or stdin, one line at a time.
// Return one space-delimited token at a time.
// Cache the remainder of the line.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

enum TokenSource {
    Stdin,
    Stream(BufReader<File>),
}
pub struct Tokenizer {
    source: TokenSource,
    line: String,             // the text of the current line
    tokens: VecDeque<String>, // a vector of tokens, successively popped off until empty and a new line is read.
    success: bool, // Set to true if the Tokenizer was properly created. Fails on file errors
}

pub impl Tokenizer {
    pub fn new(file_path: Option<&str>) -> Tokenizer {
        // Initialize a tokenizer.
        match file_path {
            None => {
                return Tokenizer {
                    source: TokenSource::Stdin,
                    line: String::new(),
                    tokens: VecDeque::new(),
                    success: true,
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
                        }
                    }
                    Err(_) => {
                        println!("File not able to be opened: {:?}", file_path);
                        return Tokenizer {
                            source: TokenSource::Stdin,
                            line: String::new(),
                            tokens: VecDeque::new(),
                            success: false,
                        };
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
                    println!("Got some values: {:?}", self.line);
                    self.tokenize();
                    println!("tokens are: {:?}", self.tokens);
                    return true;
                }
            }
            TokenSource::Stream(ref mut file) => {
                // Read from a file. TokenSource is a BufReader
                println!("get_line reading from file");
                if let Err(_) = &file.read_line(&mut self.line) {
                    return false;
                } else {
                    println!("self.line: {:?}", self.line);
                    return true;
                }
            }
        }
    }

    fn tokenize(&mut self) -> bool {
        // Tokenize the current line, returning true if successful
        // Only fails if the line is empty
        //
        println!("tokenize");
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
        println!("get_token");
        if self.tokens.is_empty() {
            println!("no more tokens; need a new line");
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
                println!("unable to get new line");
                return None;
            }
        } else {
            return self.tokens.pop_front();
        }
    }
}
