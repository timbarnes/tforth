//The tForth interpreter struct and implementation

use std::collections::HashMap;
use std::io::{self, Write};

use crate::messages::{DebugLevel, Msg};
use crate::reader::Reader;
use crate::tokenizer::{BranchInfo, ForthToken, Tokenizer};

#[derive(Debug)]
pub struct ForthInterpreter {
    pub stack: Vec<i64>, // the numeric stack, currently integers
    pub defined_words: HashMap<String, Vec<ForthToken>>, // the dictionary: keys (words) and their definitions
    pub variable_stack: Vec<i64>,                        // where variables are stored
    pub defined_variables: HashMap<String, i64>,         // separate hashmap for variables
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
                variable_stack: Vec::new(),
                defined_variables: HashMap::new(),
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
                    .debug("execute_token", "operator is", &self.token);
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
                        .debug("compile_token", "Pushing", &self.token);
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
                            if let Some((word, place)) = branch_stack.pop() {
                                self.new_word_definition[place] = ForthToken::Branch(
                                    BranchInfo::new(word.to_string(), idx - place, true),
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
            ForthToken::Forward(info) => {
                // need a better way to capture forward and branch types' unique behaviors
                match info.word.as_str() {
                    "(" => {} // ignore comments
                    ".\"" => {
                        let tail = &info.tail[..info.tail.len()];
                        println!("{}", tail);
                    }
                    "s\"" => {
                        let txt = &info.tail;
                        self.text = info.tail[..txt.len()].to_owned();
                    }
                    "variable" => {
                        let index = self.variable_stack.len();
                        self.variable_stack.push(0); // create the location for the new variable
                        self.defined_variables
                            .insert(info.tail.clone(), index as i64);
                        self.msg_handler.warning(
                            "execute_token",
                            "Dealing with a variable called",
                            info.tail.clone(),
                        );
                    }
                    _ => (),
                }
            }
            /*             ForthToken::Text(txt) | ForthToken::Comment(txt) => {
                           self.msg_handler.error(
                               "execute_token",
                               "Should not have Text or Comment tokens",
                               txt.as_str(),
                           );
                           self.abort_flag = true;
                       }
            */
            ForthToken::Branch(info) => {
                match info.word.as_str() {
                    // runtime semantics
                    "if" => {
                        if !self.stack_underflow("if", 1) {
                            let b = self.stack.pop();
                            if b.unwrap() == 0 {
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
                    "mod" => {
                        if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                            self.stack.push(b % a);
                        } else {
                            self.msg_handler.error("MOD", "Stack Underflow", "")
                        }
                    }
                    "<" => {
                        if self.stack_underflow("<", 2) {
                            self.abort_flag = true;
                        } else {
                            let l = self.stack.len() - 1;
                            let result = if self.stack[l - 1] < self.stack[l] {
                                -1
                            } else {
                                0
                            };
                            self.stack.pop();
                            self.stack.pop();
                            self.stack.push(result);
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
                                    self.stack.push(if a == b { -1 } else { 0 });
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
                            self.stack.push(if a < 0 { -1 } else { 0 });
                        }
                    }
                    ".s" => {
                        // print stack contents
                        println!("{:?}", self.stack);
                    }
                    ".s\"" => {
                        // print the saved string
                        println!("{:?}", self.text);
                    }
                    "echo" => {
                        if !self.stack_underflow("echo", 1) {
                            let n = self.stack.pop();
                            match n {
                                Some(n) => {
                                    if n > 0 && n < 128 {
                                        let c = n as u8 as char;
                                        print!("{}", c);
                                    } else {
                                        self.msg_handler.error("ECHO", "Arg out of range", n);
                                    }
                                }
                                None => {}
                            }
                        }
                    }
                    "flush" => {
                        // flush the stdout buffer to the terminal
                        io::stdout().flush().unwrap();
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
                                .warning("SWAP", "Too few elements on stack.", "");
                        }
                    }
                    "over" => {
                        if self.stack_underflow("OVER", 2) {
                            self.abort_flag = true;
                        } else {
                            let item = self.stack.get(self.stack.len() - 2);
                            match item {
                                Some(item) => {
                                    self.stack.push(*item);
                                }
                                None => {
                                    self.abort_flag = true;
                                }
                            }
                        }
                    }
                    "rot" => {
                        if self.stack_underflow("OVER", 3) {
                            self.abort_flag = true;
                        } else {
                            let top_index = self.stack.len() - 1;
                            let top = self.stack[top_index - 2];
                            let middle = self.stack[top_index];
                            let bottom = self.stack[top_index - 1];
                            self.stack[top_index - 2] = bottom;
                            self.stack[top_index - 1] = middle;
                            self.stack[top_index] = top;
                        }
                    }
                    "and" => {
                        if !self.stack_underflow("AND", 2) {
                            if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                                self.stack.push(a & b);
                            }
                        }
                    }
                    "or" => {
                        if !self.stack_underflow("OR", 2) {
                            if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                                self.stack.push(a | b);
                            }
                        }
                    }
                    "@" => {
                        if !self.stack_underflow("@", 1) {
                            if let Some(adr) = self.stack.pop() {
                                let address = adr.max(0) as usize;
                                if address < self.variable_stack.len() {
                                    self.stack.push(self.variable_stack[address]);
                                } else {
                                    self.msg_handler.error("@", "Bad variable address", adr);
                                }
                            }
                        }
                    }
                    "!" => {
                        if !self.stack_underflow("!", 2) {
                            if let (Some(addr), Some(val)) = (self.stack.pop(), self.stack.pop()) {
                                let address = addr.max(0) as usize;
                                if address < self.variable_stack.len() {
                                    self.variable_stack[address] = val;
                                }
                            }
                        }
                    }
                    "abort" => {
                        // empty the stack, reset any pending operations, and return to the prompt
                        self.msg_handler
                            .warning("ABORT", "Terminating execution", "");
                        self.stack.clear();
                        self.parser.clear();
                        self.abort_flag = true;
                    }
                    "words" => {
                        for (key, _) in self.defined_words.iter() {
                            print!("{key} ");
                        }
                        println!("");
                    }
                    "seeall" => {
                        for (key, definition) in self.defined_words.iter() {
                            self.word_see(key, definition);
                        }
                        for (key, index) in self.defined_variables.iter() {
                            self.variable_see(key, *index);
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
                    "dbg" => match self.stack.pop() {
                        Some(0) => self.msg_handler.set_level(DebugLevel::Errors),
                        Some(1) => self.msg_handler.set_level(DebugLevel::Warnings),
                        Some(2) => self.msg_handler.set_level(DebugLevel::Info),
                        _ => self.msg_handler.set_level(DebugLevel::Debug),
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
                } else if self.defined_variables.contains_key(word_name) {
                    //  check for a variable
                    self.stack.push(self.defined_variables[word_name]); // push the index on the stack
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

    pub fn load_file(&mut self, path: &str) -> bool {
        // read in a file of forth code using the provided path
        // returns false in case of error
        // does not modify self.text
        let full_path = std::fs::canonicalize(path);
        match full_path {
            Ok(full_path) => {
                // path is good
                // make a new reader (it will be swapped with self.parser.reader)
                let reader = Reader::new(Some(&full_path), "", "", Msg::new());
                match reader {
                    Some(mut previous_reader) => {
                        std::mem::swap(&mut previous_reader, &mut self.parser.reader);
                        loop {
                            if self.process_token() {
                                self.msg_handler.debug("loaded", "processed", &self.token);
                            } else {
                                self.msg_handler
                                    .debug("loaded", "No more tokens to read", "");
                                break;
                            }
                        }
                        std::mem::swap(&mut self.parser.reader, &mut previous_reader);
                        return true;
                    }
                    None => {
                        self.abort_flag = true;
                        self.msg_handler
                            .error("loaded", "Failed to create new reader", "");
                        return false;
                    }
                }
            }
            Err(error) => {
                self.msg_handler
                    .error("loaded", error.to_string().as_str(), self.text.as_str());
                self.abort_flag = true;
                return false;
            }
        }
    }

    fn loaded(&mut self) {
        // Load a file of forth code. Initial implementation is not intended to be recursive.
        // attempt to open the file, return an error if not possible
        self.load_file(self.text.clone().as_str());
    }

    fn variable_see(&self, name: &str, index: i64) {
        let idx = index.max(0) as usize;
        let value = self.variable_stack[idx];
        println!("Variable {name}: {value}");
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
                ForthToken::Forward(info) => {
                    print!("{} {}", info.word, info.tail);
                }
                ForthToken::Empty => print!("ForthToken::Empty "),
            }
        }
        println!(";");
    }
}
