// Parse a line of text, returning ForthTokens.
// Use Reader::get_line() to get a line of text
// Account for multi-line strings

use crate::messages::Msg;
use crate::reader::Reader;
use crate::utility;

#[derive(Debug, Clone)]
pub enum ForthToken {
    Integer(i64),     // the token is an integer, stored here
    Operator(String), // the token is an operator
    Text(String),     // the token is a text string
    Comment(String),  // an inline comment e.g. word stack signature
    Float(f64),       // a floating point number
    VarInt(String),   // the name of an integer variable (stored in the dictionary)
    Empty,            // the line was empty
}

#[derive(Debug)]
enum TokenType {
    Blank,
    Executable, // Words and numbers
    Text,
    Comment,
}

#[derive(Debug)]
pub struct Tokenizer {
    line: String,
    token_string: String,
    reader: Reader,
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

    pub fn get_token(&mut self) -> Option<ForthToken> {
        // Return the token or None
        // trim the token text off the front of self.line
        let token_type = self.get_token_text();
        match token_type {
            Some(token_type) => {
                match token_type {
                    TokenType::Comment => {
                        return Some(ForthToken::Comment(self.token_string.to_string()));
                    }
                    TokenType::Text => {
                        return Some(ForthToken::Text(self.token_string.to_string()));
                    }
                    TokenType::Executable => {
                        if utility::is_integer(self.token_string.as_str()) {
                            return Some(ForthToken::Integer(self.token_string.parse().unwrap()));
                        } else if utility::is_float(self.token_string.as_str()) {
                            return Some(ForthToken::Float(self.token_string.parse().unwrap()));
                        } else {
                            return Some(ForthToken::Operator(self.token_string.to_string()));
                        }
                    }
                    TokenType::Blank => {
                        return Some(ForthToken::Empty); // represents an empty line
                    }
                }
            }
            None => {
                return None;
            }
        }
    }

    fn get_token_text(&mut self) -> Option<TokenType> {
        // Get the full text for a token, recursing if necessary for multiline tokens (text string or comment)
        let mut multiline = false; // to drive the prompt
        let mut token_type = TokenType::Blank;
        self.token_string.clear();
        let mut chars_used = 0;
        'per_line: loop {
            // We explicitly break out when we have a complete token
            if self.line.is_empty() {
                let line = self.reader.get_line(multiline);
                match line {
                    Some(line) => {
                        self.line = line;
                    }
                    None => {
                        return None;
                    }
                }
            }
            'scan: for c in self.line.chars() {
                match token_type {
                    TokenType::Blank => {
                        match c {
                            ' ' => {
                                // skip over blanks
                                chars_used += 1;
                            }
                            '\"' => {
                                self.token_string.push(c); // save the quote
                                token_type = TokenType::Text;
                                chars_used += 1;
                            }
                            '(' => {
                                self.token_string.push(c); // save the paren
                                token_type = TokenType::Comment;

                                chars_used += 1;
                            }
                            '\n' => {
                                // We're finished
                                self.line.clear();
                                break 'per_line;
                            }
                            _ => {
                                self.token_string.push(c);
                                chars_used += 1;
                                token_type = TokenType::Executable;
                            }
                        }
                    }
                    TokenType::Text | TokenType::Comment => {
                        match c {
                            '\"' | ')' => {
                                self.token_string.push(c);
                                chars_used += 1;
                                break 'per_line;
                            }
                            '\n' => {
                                // partial string is complete
                                self.token_string.push(c);
                                chars_used = 0;
                                self.line.clear();
                                multiline = true;
                                break 'scan;
                            }
                            _ => {
                                self.token_string.push(c);
                                chars_used += 1;
                            }
                        }
                    }
                    TokenType::Executable => match c {
                        ' ' | '\n' => {
                            chars_used += 1;
                            break 'per_line;
                        }
                        _ => {
                            self.token_string.push(c);
                            chars_used += 1;
                            multiline = false;
                        }
                    },
                }
            }
        }
        self.line = self.line[chars_used..].to_string(); // eliminate the characters that have been used
        self.msg.info(
            "get_token_text",
            "Token, type, line, chars_used",
            (&token_type, &self.token_string, &self.line, chars_used),
        );

        return Some(token_type);
    }
}
