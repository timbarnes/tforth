// Parse a line of text, returning ForthTokens.
// Use Reader::get_line() to get a line of text
// Account for multi-line strings

use crate::messages::Msg;
use crate::reader::Reader;
//use crate::utility;

const BRANCHES: [&str; 8] = [
    "if", "else", "then", "begin", "loop", "until", "repeat", "+loop",
];
const FORWARDS: [(&str, &str); 5] = [
    ("(", ")"),
    ("s\"", "\""),
    (".\"", "\""),
    ("see", " \t\n"),
    ("variable", " \t\n"),
];

#[derive(Debug, Clone)]
pub enum ForthToken {
    Integer(i64),         // the token is an integer, stored here
    Operator(String),     // the token is an operator
    Branch(BranchInfo),   // branch
    Forward(ForwardInfo), // a read_ahead token (string, comment etc.)
    Float(f64),           // a floating point number
    Empty,                // the line was empty
}

#[derive(Debug, Clone)]
pub struct ForwardInfo {
    pub word: String,
    pub tail: String,
}

impl ForwardInfo {
    pub fn new(word: String, tail: String) -> ForwardInfo {
        ForwardInfo { word, tail }
    }
}

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub word: String,
    pub offset: usize,
    pub conditional: bool,
}

impl BranchInfo {
    pub fn new(word: String, offset: usize, conditional: bool) -> BranchInfo {
        BranchInfo {
            word,
            offset,
            conditional,
        }
    }
}

#[derive(Debug)]
pub struct Tokenizer {
    line: String,
    token_string: String,
    pub reader: Reader,
    msg: Msg,
}

impl Tokenizer {
    pub fn new(reader: Reader) -> Tokenizer {
        Tokenizer {
            line: String::new(),
            token_string: String::new(),
            reader: reader,
            msg: Msg::new(),
        }
    }

    pub fn clear(&mut self) {
        self.line.clear();
        self.token_string.clear();
    }

    pub fn get_token(&mut self) -> Option<ForthToken> {
        // Return the token or None
        // trim the token text off the front of self.line
        let token_text = self.get_token_text();
        match token_text {
            None => {
                self.msg.error("get_token", "No token string", &token_text);
                return None;
            }
            Some(text) => {
                if is_integer(&text) {
                    return Some(ForthToken::Integer(text.parse().unwrap()));
                } else if is_float(&text) {
                    return Some(ForthToken::Float(text.parse().unwrap()));
                } else if BRANCHES.contains(&text.as_str()) {
                    return Some(ForthToken::Branch(BranchInfo::new(text, 0, false)));
                } else {
                    // it's a Forward or an Operator
                    for (word, terminator) in FORWARDS {
                        if word == text {
                            match self.read_until(terminator) {
                                Some(remainder) => {
                                    return Some(ForthToken::Forward(ForwardInfo::new(
                                        text.to_owned(),
                                        format!("{remainder}"),
                                    )));
                                }
                                None => {
                                    return Some(ForthToken::Forward(ForwardInfo::new(
                                        text.to_owned(),
                                        "".to_owned(),
                                    )));
                                }
                            }
                        }
                    }
                    return Some(ForthToken::Operator(text.to_owned()));
                }
            }
        }
    }

    pub fn read_until(&mut self, terminator: &str) -> Option<String> {
        // Read from the input stream, returning a string terminating in the first occurrence
        // of  end_char.
        let mut multiline = false; // to drive the prompt
        let mut token_string = String::new();
        let mut chars_used = 0;
        loop {
            // We explicitly break out when we have a complete token
            if self.line.is_empty() {
                let line = self.reader.get_line(multiline);
                match line {
                    Some(line) => {
                        self.line = line;
                    }
                    None => {
                        return None; // Signals EOF
                    }
                }
            }
            'scan: for c in self.line.chars() {
                if terminator.contains(c) {
                    // We're done. We don't return the end_char as part of the string.
                    self.line = self.line[chars_used + 1..].to_string();
                    return Some(token_string);
                } else if c == '\n' {
                    // end of line, so break out and get another
                    token_string.push(c);
                    chars_used = 0;
                    multiline = true;
                    self.line.clear();
                    break 'scan;
                } else {
                    token_string.push(c);
                    chars_used += 1;
                }
            }
        }
    }

    fn get_token_text(&mut self) -> Option<String> {
        // Get a single word, space or \n delimited.
        let mut token_string = String::new();
        let mut chars_used = 0;
        if self.line.is_empty() {
            match self.reader.get_line(false) {
                Some(line) => {
                    self.line = line;
                    self.msg.debug(
                        "get_token_text",
                        "read a line of length",
                        format!("{:?}", self.line.len()),
                    );
                }
                None => {
                    // Reader error
                    self.msg.error("get_token_text", "reader error", "");
                    return None;
                }
            }
        }
        self.line = self.line.trim_start().to_string(); // We never need leading spaced.
        for c in self.line.chars() {
            match c {
                '\n' | '\t' | ' ' => {
                    break;
                }
                _ => {
                    token_string.push(c);
                    chars_used += 1;
                }
            }
        }
        if chars_used == 0 {
            self.msg
                .debug("get_token_text", "end of line", &token_string);
            self.line.clear();
            return self.get_token_text(); // go again
        } else {
            self.line = self.line[chars_used + 1..].to_string();
            self.msg.debug("get_token_text", "returning", &token_string);
            return Some(token_string);
        }
    }
}

pub fn is_integer(s: &str) -> bool {
    s.parse::<i64>().is_ok()
}

pub fn is_float(s: &str) -> bool {
    s.parse::<f64>().is_ok()
}
