// Read tokens from a file or stdin, one line at a time.
// Return one space-delimited token at a time.
// Cache the remainder of the line.

use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};

use crate::messages::{DebugLevel, Msg};

#[derive(Debug)]
enum Source {
    Stdin,
    Stream(BufReader<File>),
}

pub struct Reader {
    source: Source,      // Stdin or a file
    prompt: String,      // the standard prompt
    cont_prompt: String, // the continuation prompt
    msg: Msg,
}

impl fmt::Debug for Reader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Tokenizer").field(&self.source).finish()
    }
}

impl Reader {
    pub fn new(
        file_path: Option<&std::path::PathBuf>,
        prompt: &str,
        cont_prompt: &str,
        msg_handler: Msg,
    ) -> Option<Reader> {
        // Initialize a tokenizer.
        let mut message_handler = Msg::new();
        message_handler.set_level(DebugLevel::Error);
        match file_path {
            None => Some(Reader {
                source: Source::Stdin,
                prompt: prompt.to_owned(),
                cont_prompt: cont_prompt.to_owned(),
                msg: msg_handler,
            }),
            Some(filepath) => {
                let file = File::open(filepath);
                match file {
                    Ok(file) => Some(Reader {
                        source: Source::Stream(BufReader::new(file)),
                        prompt: prompt.to_owned(),
                        cont_prompt: cont_prompt.to_owned(),
                        msg: msg_handler,
                    }),
                    Err(_) => {
                        msg_handler.error(
                            "Reader::new",
                            "File not able to be opened",
                            Some(file_path),
                        );
                        None
                    }
                }
            }
        }
    }

    pub fn get_line(&mut self, current_stack: &String, multiline: bool) -> Option<String> {
        // Read a line, storing it if there is one
        // In interactive (stdin) mode, blocks until the user provides a line.
        // Returns Option(line text). None indicates the read failed.
        let mut new_line = String::new();
        let result;
        match self.source {
            Source::Stdin => {
                if multiline {
                    print!("{}", self.cont_prompt);
                } else {
                    print!("{} {}", current_stack, self.prompt);
                }
                io::stdout().flush().unwrap();
                result = io::stdin().read_line(&mut new_line);
            }
            Source::Stream(ref mut file) => result = file.read_line(&mut new_line),
        }
        match result {
            Ok(chars) => {
                if chars > 0 {
                    Some(new_line)
                } else {
                    None
                }
            }
            Err(e) => {
                self.msg
                    .error("get_line", "read_line error", Some(e.to_string()));
                None
            }
        }
    }

    pub fn read_char(&self) -> Option<char> {
        let mut buf = [0; 1];
        let mut handle = io::stdin().lock();
        let bytes_read = handle.read(&mut buf);
        match bytes_read {
            Ok(_size) => Some(buf[0] as char),
            Err(_) => None,
        }
    }
}
