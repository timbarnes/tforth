// Read tokens from a file or stdin, one line at a time.
// Return one space-delimited token at a time.
// Cache the remainder of the line.

use std::fs::File;
use std::io::{self};

struct Tokenizer {
    filename: &str,    // name of the file to read from, or "stdin"
    stream: File,      // file handle
    line: &str,        // the text of the current line
    tokens: Vec<&str>, // a vector of tokens, successively popped off until empty and a new line is read.
}

impl Tokenizer {
    pub fn new(path: &str) -> Tokenizer {
        // Initialize a tokenizer. path is a filename or stdin
        Tokenizer {
            filename: if path == "stdin" { "" } else { path },
            stream: if path == "stdin" {
                io::stdin
            } else {
                File.open(path)
            },
            line: "",
            tokens: Vec::new(),
        }
    }

    fn read_line(&self) -> Option(&str) {
        // Read a line, returning it if there is one
        // In interactive (stdin) mode, blocks until the user provides a line.
        if let line = self.stream.read_line() {
            Some(line)
        } else {
            None
        }
    }

    pub fn get_token(&mut self) -> Option(&str) {
        // Returns the next token from the stream, if there is one
    }
}
