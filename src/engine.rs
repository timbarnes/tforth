//The Tforth interpreter struct and implementation

use std::collections::HashMap;

use crate::messages::{DebugLevel, Msg};
use crate::reader::Reader;
use crate::tokenizer::{ForthToken, Tokenizer};

#[derive(Debug)]
pub struct ForthInterpreter {
    pub stack: Vec<i64>, // the numeric stack, currently integers
    pub defined_words: HashMap<String, Vec<ForthToken>>, // the dictionary: keys (words) and their definitions
    text: String,                                        // the current s".."" string
    file_mode: FileMode,
    compile_mode: bool, // true if compiling a word
    abort_flag: bool,   // true if abort has been called
    exit_flag: bool,    // set when the "bye" word is executed.
    pub msg_handler: Msg,
    parser: Tokenizer,
    new_word_name: String,
    new_word_definition: Vec<ForthToken>,
    token: ForthToken,
}

#[derive(Debug)]
enum FileMode {
    // used for file I/O
    ReadWrite,
    ReadOnly,
    Unset,
}

impl ForthInterpreter {
    // ForthInterpreter struct implementations
    pub fn new(main_prompt: &str, multiline_prompt: &str) -> ForthInterpreter {
        ForthInterpreter {
            stack: Vec::new(),
            defined_words: HashMap::new(),
            text: String::new(),
            file_mode: FileMode::Unset,
            compile_mode: false,
            abort_flag: false,
            exit_flag: false,
            msg_handler: Msg::new(),
            parser: Tokenizer::new(Reader::new(None, main_prompt, multiline_prompt)),
            new_word_name: String::new(),
            new_word_definition: Vec::new(),
            token: ForthToken::Empty,
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

    pub fn get_compile_mode(&self) -> bool {
        self.compile_mode
    }

    fn set_compile_mode(&mut self, state: bool) {
        self.compile_mode = state;
    }

    fn stack_underflow(&self, op: &str, n: usize) -> bool {
        if self.stack.len() < n {
            self.msg_handler.error(op, "Stack underflow", "");
            true
        } else {
            false
        }
    }

    pub fn process_token(&mut self) -> bool {
        let new_token = self.parser.get_token(); // Prompt if necessary, return a token
        match new_token {
            Some(new_token) => {
                self.msg_handler
                    .info("execute_token", "operator is", &self.token);
                self.token = new_token;
                match self.token {
                    ForthToken::Empty => {
                        return true; // An empty line
                    }
                    _ => {
                        // Any valid token
                        if self.get_compile_mode() {
                            self.compile_token();
                        } else {
                            // we're in immediate mode
                            self.execute_token();
                        }
                    }
                }
                return true;
            }
            None => {
                return false;
            }
        }
    }

    fn compile_token(&mut self) {
        // We're in compile mode: compile the new word
        let tok = &self.token;
        match tok {
            ForthToken::Operator(tstring) => {
                if tstring == ";" {
                    // we are at the end of the definition
                    self.defined_words
                        .insert(self.new_word_name.clone(), self.new_word_definition.clone());
                    self.new_word_name.clear();
                    self.new_word_definition.clear();
                    self.set_compile_mode(false);
                } else if self.new_word_name.is_empty() {
                    // We've found the word name
                    self.new_word_name = tstring.to_string();
                } else if tstring == ":" {
                    self.msg_handler
                        .warning("compile_token", "Illegal inside definition", ":");
                } else {
                    // push the new token onto the definition
                    self.msg_handler
                        .warning("compile_token", "Pushing", &self.token);
                    self.new_word_definition.push(self.token.clone());
                }
            }
            _ => {
                // Text, integer, float, comment all go into the new word definition
                self.new_word_definition.push(self.token.clone());
            }
        }
    }

    fn execute_token(&mut self) -> bool {
        // Immediate mode:
        // Execute a defined token
        match &self.token {
            ForthToken::Empty => return false,
            ForthToken::Integer(num) => {
                self.stack.push(*num);
            }
            ForthToken::Float(_num) => {
                // stack needs to support floats, ints, and pointers
                // self.stack.push(num);
            }
            ForthToken::Text(txt) => {
                // save the string
                self.text = txt.clone();
            }
            ForthToken::VarInt(name) => {
                self.msg_handler
                    .warning("execute_token", "VarInt not implemented", name);
            }
            ForthToken::Comment(_txt) => {
                () // skip over the comment
            }
            ForthToken::Operator(op) => {
                match op.as_str() {
                    "+" => {
                        if self.stack_underflow("+", 2) {
                            ()
                        } else {
                            if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                                self.stack.push(a + b);
                            }
                        }
                    }
                    "-" => {
                        if self.stack_underflow("-", 2) {
                            ()
                        } else {
                            if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                                self.stack.push(b - a);
                            }
                        }
                    }
                    "*" => {
                        if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                            self.stack.push(a * b);
                        } else {
                            self.msg_handler.error("*", "Stack Underflow", "")
                        }
                    }
                    "/" => {
                        if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                            self.stack.push(b / a);
                        } else {
                            self.msg_handler.error("/", "Stack Underflow", "")
                        }
                    }
                    "." => {
                        if let Some(a) = self.stack.pop() {
                            println!("{a}");
                        } else {
                            self.msg_handler.error(".", "Stack Underflow", "")
                        }
                    }
                    "true" => {
                        self.stack.push(0);
                    }
                    "false" => {
                        self.stack.push(-1);
                    }
                    "=" => {
                        if self.stack_underflow("=", 2) {
                            ()
                        } else {
                            if let Some(a) = self.stack.pop() {
                                if let Some(b) = self.stack.pop() {
                                    self.stack.push(if a == b { 0 } else { 1 });
                                }
                            }
                        }
                    }
                    "0=" => {
                        if let Some(a) = self.stack.pop() {
                            self.stack.push(if a == 0 { 0 } else { 1 });
                        }
                    }
                    "0<" => {
                        if let Some(a) = self.stack.pop() {
                            self.stack.push(if a < 0 { 0 } else { 1 });
                        }
                    }
                    ".s" => {
                        println!("{:?}", self.stack);
                    }
                    ".\"" => {
                        println!("{:?}", self.text);
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
                    "abort" => {
                        // empty the stack, reset any pending operations, and return to the prompt
                        self.msg_handler
                            .warning("abort", "Terminating execution", "");
                        self.stack.clear();
                        self.parser.clear();
                        self.abort_flag = true;
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
                                    ForthToken::Integer(num) => print!("int:{num} "),
                                    ForthToken::Float(num) => print!("float:{num} "),
                                    ForthToken::Operator(op) => print!("op:{op} "),
                                    ForthToken::Comment(c) => print!("comment:{c} "),
                                    ForthToken::Text(txt) => print!("text:{txt} "),
                                    ForthToken::VarInt(txt) => print!("VarInt{txt}"),
                                    ForthToken::Empty => print!("ForthToken::Empty "),
                                }
                            }
                            println!(";");
                        }
                    }
                    "r/w" => {
                        self.file_mode = FileMode::ReadWrite;
                    }
                    "r/o" => {
                        self.file_mode = FileMode::ReadOnly;
                    }
                    "loaded" => {
                        self.loaded();
                    }
                    "debuglevel" => match self.stack.pop() {
                        Some(0) => self.msg_handler.set_level(DebugLevel::No),
                        Some(1) => self.msg_handler.set_level(DebugLevel::Warning),
                        _ => self.msg_handler.set_level(DebugLevel::Info),
                    },
                    "debuglevel?" => {
                        println!("DebugLevel is {:?}", self.msg_handler.get_level());
                    }
                    ":" => {
                        // Enter compile mode
                        self.set_compile_mode(true);
                    }
                    "bye" => {
                        self.set_exit_flag();
                    }
                    // Add more operators as needed
                    _ => {
                        // It must be a defined word
                        self.execute_definition();
                    }
                }
            }
        }
        return true;
    }

    fn execute_definition(&mut self) {
        // execute a word defined in forth
        // see if the word is in the dictionary.
        // if so, iterate over the definition, using execute_token()
        match &self.token {
            ForthToken::Operator(word_name) => {
                if self.defined_words.contains_key(word_name) {
                    let mut definition = self.defined_words[word_name.as_str()].clone();
                    for w in &definition {
                        if self.abort_flag {
                            definition.clear();
                            self.abort_flag = false;
                            break;
                        } else {
                            self.token = w.clone();
                            self.execute_token();
                        }
                    }
                } else {
                    self.msg_handler
                        .error("execute_definition", "Undefined word", &word_name);
                    return;
                }
            }
            _ => {
                self.msg_handler
                    .error("execute_definition", "Definition error", "");
                return;
            }
        }
    }

    fn loaded(&self) {
        // Load a file of forth code
        self.msg_handler.info(
            "loaded",
            "Attempting to load file",
            (&self.text, &self.file_mode),
        );
        // attempt to open the file, return an error if not possible
    }
}
