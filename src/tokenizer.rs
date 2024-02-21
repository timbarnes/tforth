// Parse a line of text, returning ForthTokens.
// Use Reader::get_line() to get a line of text
// Account for multi-line strings

use crate::engine::OpCode;
use crate::messages::Msg;
use crate::reader::Reader;
//use crate::utility;

const BRANCHES: [&str; 5] = ["if", "else", "then", "for", "next"];
const FORWARDS: [(&str, &str); 7] = [
    ("(", ")"),            // comment
    ("s\"", "\""),         // stored string
    (".\"", "\""),         // inline string print
    ("see", " \t\n"),      // view word definition
    ("variable", " \t\n"), // variable declaration
    ("constant", " \t\n"), // constant declaration
    ("\\", "\n"),          // comment to end of line
];

#[derive(Debug, Clone)]
pub enum ForthToken {
    Integer(i64),         // the token is an integer, stored here
    Operator(String),     // the token is an operator - either definition or builtin
    Jump(String, usize),  // branch
    Forward(ForwardInfo), // a read_ahead token (string, comment etc.)
    Definition(String, Vec<OpCode>),
    Builtin(String, usize), // name and op code for the builtin
    Variable(String, i64),
    Constant(String, i64),
    StringVar(String, String), // a text string variable
    Float(f64),                // a floating point number
    Empty,                     // the line was empty
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

#[derive(Debug)]
pub struct Tokenizer {
    line: String,
    pub reader: Reader,
    branch_counter: usize,
    msg: Msg,
}

impl Tokenizer {
    pub fn new(reader: Reader) -> Tokenizer {
        Tokenizer {
            line: String::new(),
            reader,
            branch_counter: 0,
            msg: Msg::new(),
        }
    }

    pub fn get_token(&mut self, current_stack: &String) -> Option<ForthToken> {
        // Return the token or None
        // trim the token text off the front of self.line
        let token_text = self.get_token_text(current_stack);
        match token_text {
            None => {
                // self.msg.error("get_token", "No token string", &token_text);
                None
            }
            Some(text) => {
                if u_is_integer(&text) {
                    Some(ForthToken::Integer(text.parse().unwrap()))
                } else if is_float(&text) {
                    Some(ForthToken::Float(text.parse().unwrap()))
                } else if BRANCHES.contains(&text.as_str()) {
                    self.branch_counter += 1;
                    Some(ForthToken::Jump(text, 0))
                } else {
                    // it's a Forward or an Operator
                    for (word, terminator) in FORWARDS {
                        if word == text {
                            match self.read_until(terminator) {
                                Some(remainder) => {
                                    return Some(ForthToken::Forward(ForwardInfo::new(
                                        text.to_owned(),
                                        remainder.to_string(),
                                    )))
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
                    Some(ForthToken::Operator(text.to_owned()))
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
                let line = self.reader.get_line(&"".to_owned(), multiline);
                match line {
                    Some(line) => self.line = line,

                    None => return None, // Signals EOF
                }
            }
            'scan: for c in self.line.chars() {
                if chars_used > 0 && terminator.contains(c) {
                    self.line = self.line[chars_used + 1..].to_string();
                    token_string.push(c);
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

    fn get_token_text(&mut self, current_stack: &String) -> Option<String> {
        // Get a single word, space or \n delimited.
        let mut token_string = String::new();
        let mut chars_used = 0;
        if self.line.is_empty() {
            match self.reader.get_line(current_stack, false) {
                Some(line) => {
                    self.line = line;
                    self.msg.debug(
                        "get_token_text",
                        "read a line of length",
                        Some(format!("{:?}", self.line.len())),
                    );
                }
                None => return None,
            }
        }
        self.line = self.line.trim_start().to_string(); // We never need leading spaces.
        for c in self.line.chars() {
            match c {
                '\n' | '\t' | ' ' => break,
                _ => {
                    token_string.push(c);
                    chars_used += 1;
                }
            }
        }
        if chars_used == 0 {
            self.msg
                .debug("get_token_text", "end of line", Some(&token_string));
            self.line.clear();
            self.get_token_text(current_stack) // go again
        } else {
            self.line = self.line[chars_used..].to_string();
            self.msg
                .debug("get_token_text", "returning", Some(&token_string));
            Some(token_string)
        }
    }
}

pub fn u_is_integer(s: &str) -> bool {
    s.parse::<i64>().is_ok()
}

pub fn is_float(s: &str) -> bool {
    s.parse::<f64>().is_ok()
}
