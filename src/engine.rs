//The Tforth interpreter struct and implementation

use std::collections::HashMap;
use std::fs::File;

use crate::messages::{DebugLevel, Msg};
use crate::reader::Reader;
use crate::tokenizer::{BranchInfo, ForthToken, Tokenizer};

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
        if let Some(reader) = Reader::new(None, main_prompt, multiline_prompt, Msg::new()) {
            let parser = Tokenizer::new(reader);
            ForthInterpreter {
                stack: Vec::new(),
                defined_words: HashMap::new(),
                text: String::new(),
                file_mode: FileMode::Unset,
                compile_mode: false,
                abort_flag: false,
                exit_flag: false,
                msg_handler: Msg::new(),
                parser: parser,
                new_word_name: String::new(),
                new_word_definition: Vec::new(),
                token: ForthToken::Empty,
            }
        } else {
            panic!("unable to create reader");
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
                            self.execute_token(0, false);
                        }
                    }
                }
                return true;
            }
            None => {
                return false; // Signals end of file
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
                    self.calculate_branches();
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
                        .info("compile_token", "Pushing", &self.token);
                    self.new_word_definition.push(self.token.clone());
                }
            }
            _ => {
                // Text, integer, float, comment all go into the new word definition
                self.new_word_definition.push(self.token.clone());
            }
        }
    }

    fn calculate_branches(&mut self) {
        // replace words that incorporate branches with ForthToken::Branch
        // and set up offsets as required.
        let mut branch_stack = Vec::<(&str, usize)>::new();
        let mut idx = 0; // points to the current token
        while idx < self.new_word_definition.len() {
            let cur_token = &self.new_word_definition[idx];
            match cur_token {
                ForthToken::Branch(branch_info) => {
                    match branch_info.word.as_str() {
                        "if" => {
                            // put the info on the stack
                            branch_stack.push(("if", idx));
                        }
                        "else" => {
                            // pop stack, insert new ZeroEqual Branch token with offset
                            if let Some((_word, place)) = branch_stack.pop() {
                                self.new_word_definition[place] = ForthToken::Branch(
                                    BranchInfo::new("if".to_string(), idx - place, true),
                                );
                            }
                            branch_stack.push(("else", idx));
                        }
                        "then" => {
                            // pop stack, insert new Unconditional Branch token with offset
                            if let Some((_word, place)) = branch_stack.pop() {
                                self.new_word_definition[place] = ForthToken::Branch(
                                    BranchInfo::new("else".to_string(), idx - place, true),
                                );
                            }
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
            idx += 1;
        }
    }

    fn execute_token(&mut self, mut program_counter: usize, mut jumped: bool) -> (usize, bool) {
        // Execute a defined token
        program_counter += 1; // base assumption is we're processing one word
        match &self.token {
            ForthToken::Empty => return (program_counter, false),
            ForthToken::Integer(num) => {
                self.stack.push(*num);
            }
            ForthToken::Float(_num) => {
                // stack needs to support floats, ints, and pointers
                // self.stack.push(num);
            }
            ForthToken::Text(txt) => {
                // save the string after removing the quotes and leading space
                self.text = txt[2..txt.len() - 1].to_owned();
            }
            ForthToken::VarInt(name) => {
                self.msg_handler
                    .warning("execute_token", "VarInt not implemented", name);
            }
            ForthToken::Comment(_txt) => {
                () // skip over the comment
            }
            ForthToken::Branch(info) => {
                match info.word.as_str() {
                    // runtime semantics
                    "if" => {
                        if !self.stack_underflow("if", 1) {
                            let b = self.stack.pop();
                            if b.unwrap() != 0 {
                                program_counter += info.offset;
                                jumped = true;
                            } else {
                                jumped = false;
                            }
                        }
                    }
                    "else" => {
                        if jumped {
                            jumped = false
                        } else {
                            program_counter += info.offset;
                            jumped = true;
                        }
                    }
                    "then" => {
                        jumped = false;
                    }
                    _ => (),
                }
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
                    "seeall" => {
                        for (key, definition) in self.defined_words.iter() {
                            self.word_see(key, definition);
                        }
                    }
                    "see" => {
                        // ( "word name" -- ) print a word's definition
                        match self.defined_words.get(self.text.as_str()) {
                            Some(definition) => {
                                self.word_see(self.text.as_str(), definition);
                            }
                            None => {
                                self.msg_handler
                                    .error("see", "Word not found", self.text.as_str());
                            }
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
        return (program_counter, jumped);
    }

    fn execute_definition(&mut self) {
        // execute a word defined in forth
        // see if the word is in the dictionary.
        // if so, iterate over the definition, using execute_token()
        let mut program_counter: usize = 0;
        let mut jumped = false;
        match &self.token {
            ForthToken::Operator(word_name) => {
                if self.defined_words.contains_key(word_name) {
                    let mut definition = self.defined_words[word_name.as_str()].clone();
                    while program_counter < definition.len() {
                        if self.abort_flag {
                            definition.clear();
                            self.abort_flag = false;
                            break;
                        } else {
                            self.token = definition[program_counter].clone();
                            (program_counter, jumped) = self.execute_token(program_counter, jumped);
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

    fn loaded(&mut self) {
        // Load a file of forth code. Initial implementation is not intended to be recursive.
        self.msg_handler.info(
            "loaded",
            "Attempting to load file",
            (&self.text, &self.file_mode),
        );
        // attempt to open the file, return an error if not possible
        let load_file = File::open(self.text.as_str());
        match load_file {
            Ok(_handle) => {
                // success: read the file
                // make a new reader (it will be swapped with self.parser.reader)
                let reader = Reader::new(Some(self.text.as_str()), "", "", Msg::new());
                match reader {
                    Some(mut previous_reader) => {
                        std::mem::swap(&mut previous_reader, &mut self.parser.reader);
                        loop {
                            if self.process_token() {
                                self.msg_handler.warning("loaded", "processed", &self.token);
                            } else {
                                self.msg_handler
                                    .warning("loaded", "No more tokens to read", "");
                                break;
                            }
                        }
                        std::mem::swap(&mut self.parser.reader, &mut previous_reader);
                        return;
                    }
                    None => {
                        self.abort_flag = true;
                        self.msg_handler
                            .error("loaded", "Failed to create new reader", "");
                    }
                }
            }
            Err(error) => {
                self.msg_handler
                    .error("loaded", error.to_string().as_str(), self.text.as_str());
                self.abort_flag = true;
            }
        }
    }

    fn word_see(&self, name: &str, definition: &Vec<ForthToken>) {
        print!(": {name} ");
        for word in definition {
            match word {
                ForthToken::Integer(num) => print!("{num} "),
                ForthToken::Float(num) => print!("f{num} "),
                ForthToken::Operator(op) => print!("{op} "),
                ForthToken::Branch(info) => {
                    print!("{}:{}:{} ", info.word, info.offset, info.conditional);
                }
                ForthToken::Comment(c) => print!("{c} "),
                ForthToken::Text(txt) => print!("{txt} "),
                ForthToken::VarInt(txt) => print!("{txt} "),
                ForthToken::Empty => print!("ForthToken::Empty "),
            }
        }
        println!(";");
    }
}
