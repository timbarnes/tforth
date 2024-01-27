// Read tokens from a file or stdin, one line at a time.
// Return one space-delimited token at a time.
// Cache the remainder of the line.

use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

use crate::messages::{DebugLevel, Msg};

#[derive(Debug)]
enum Source {
    Stdin,
    Stream(BufReader<File>),
}

pub struct Reader {
    source: Source,      // Stdin or a file
    line: String,        // the text of the current line
    prompt: String,      // the standard prompt
    cont_prompt: String, // the continuation prompt
    msg: Msg,
}

impl fmt::Debug for Reader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Tokenizer")
            .field(&self.source)
            .field(&self.line)
            .finish()
    }
}

impl Reader {
    pub fn new(file_path: Option<&str>, prompt: &str, cont_prompt: &str) -> Reader {
        // Initialize a tokenizer.
        let mut message_handler = Msg::new();
        message_handler.set_level(DebugLevel::No);
        match file_path {
            None => {
                return Reader {
                    source: Source::Stdin,
                    line: String::new(),
                    prompt: prompt.to_owned(),
                    cont_prompt: cont_prompt.to_owned(),
                    msg: message_handler,
                }
            }
            Some(filepath) => {
                let file = File::open(filepath);
                match file {
                    Ok(file) => {
                        return Reader {
                            source: Source::Stream(BufReader::new(file)),
                            line: String::new(),
                            prompt: prompt.to_owned(),
                            cont_prompt: cont_prompt.to_owned(),
                            msg: message_handler,
                        }
                    }
                    Err(_) => {
                        let tkn = Reader {
                            source: Source::Stdin,
                            line: String::new(),
                            prompt: prompt.to_owned(),
                            cont_prompt: cont_prompt.to_owned(),
                            msg: message_handler,
                        };
                        tkn.msg
                            .error("Reader::new", "File not able to be opened", file_path);
                        return tkn;
                    }
                }
            }
        }
    }

    pub fn get_line(&mut self, multiline: bool) -> Option<String> {
        // Read a line, storing it if there is one
        // In interactive (stdin) mode, blocks until the user provides a line.
        // Returns Option(line text). None indicates the read failed.
        match self.source {
            Source::Stdin => {
                // Issue prompt
                self.line.clear();
                if multiline {
                    print!("{}", self.cont_prompt);
                } else {
                    print!("{}", self.prompt);
                }
                io::stdout().flush().unwrap();
                // Read from Stdin
                if let Err(_) = io::stdin().read_line(&mut self.line) {
                    return None;
                } else {
                    self.msg.info("get_line", "Got some values", &self.line);
                    return Some(self.line.clone());
                }
            }
            Source::Stream(ref mut file) => {
                // Read from a file. TokenSource is a BufReader. No prompts
                self.msg.info("get_line", "Reading from file", "");
                if let Err(_) = &file.read_line(&mut self.line) {
                    return None;
                } else {
                    self.msg.info("self.line", "Text was", &self.line);
                    return Some(self.line.clone());
                }
            }
        }
    }
}
