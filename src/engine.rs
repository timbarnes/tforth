//The tForth interpreter struct and implementation

use std::collections::HashMap;
use std::io::{self, Write};

use crate::doc;
use crate::messages::{DebugLevel, Msg};
use crate::reader::Reader;
use crate::tokenizer::{BranchInfo, ForthToken, Tokenizer};

#[derive(Debug)]
pub struct ForthInterpreter {
    pub stack: Vec<i64>, // the numeric stack, currently integers
    pub defined_words: HashMap<String, Vec<ForthToken>>, // the dictionary: keys (words) and their definitions
    pub variable_stack: Vec<i64>,                        // where variables are stored
    pub defined_variables: HashMap<String, i64>,         // separate hashmap for variables
    pub control_stack: Vec<i64>,                         // for do loops etc.
    builtin_doc: HashMap<String, String>,                // doc strings for built-in words
    text: String,                                        // the current s".."" string
    file_mode: FileMode,
    compile_mode: bool, // true if compiling a word
    abort_flag: bool,   // true if abort has been called
    exit_flag: bool,    // set when the "bye" word is executed.
    pub msg: Msg,
    parser: Tokenizer,
    new_word_name: String,
    new_word_definition: Vec<ForthToken>,
    token: ForthToken,
    show_stack: bool, // show the stack at the completion of a line of interaction
    step_mode: bool,
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
        let doc_strings = doc::build_doc_strings();
        if let Some(reader) = Reader::new(None, main_prompt, multiline_prompt, Msg::new()) {
            let parser = Tokenizer::new(reader);
            ForthInterpreter {
                stack: Vec::new(),
                defined_words: HashMap::new(),
                text: String::new(),
                variable_stack: Vec::new(),
                defined_variables: HashMap::new(),
                control_stack: Vec::new(),
                builtin_doc: doc_strings,
                file_mode: FileMode::Unset,
                compile_mode: false,
                abort_flag: false,
                exit_flag: false,
                msg: Msg::new(),
                parser,
                new_word_name: String::new(),
                new_word_definition: Vec::new(),
                token: ForthToken::Empty,
                show_stack: false,
                step_mode: false,
            }
        } else {
            panic!("unable to create reader");
        }
    }

    pub fn set_abort_flag(&mut self, v: bool) {
        self.abort_flag = v;
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
            self.msg.error(op, "Stack underflow", "");
            true
        } else {
            false
        }
    }

    fn pop_one(&mut self, word: &str) -> Option<i64> {
        let val = self.stack.pop();
        match val {
            Some(value) => Some(value),
            None => {
                self.msg.error(word, "Stack underflow", "");
                None
            }
        }
    }

    fn pop_two(&mut self, word: &str) -> Option<(i64, i64)> {
        let (val1, val2) = (self.stack.pop(), self.stack.pop());
        match val1 {
            Some(value1) => match val2 {
                Some(value2) => Some((value1, value2)),
                None => None,
            },
            None => {
                self.msg.error(word, "Stack underflow", "");
                None
            }
        }
    }

    pub fn process_token(&mut self) -> bool {
        let new_token = self.parser.get_token(&self.get_stack()); // Prompt if necessary, return a token
        match new_token {
            Some(new_token) => {
                self.msg.debug("execute_token", "operator is", &self.token);
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
                true
            }
            None => {
                false // Signals end of file
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
                    self.msg
                        .warning("compile_token", "Illegal inside definition", ":");
                } else {
                    // push the new token onto the definition
                    self.msg.debug("compile_token", "Pushing", &self.token);
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
            if let ForthToken::Branch(branch_info) = cur_token {
                match branch_info.word.as_str() {
                    "if" => {
                        // put the info on the stack
                        branch_stack.push(("if", idx));
                    }
                    "else" => {
                        // pop stack, insert new ZeroEqual Branch token with offset
                        if let Some((_word, place)) = branch_stack.pop() {
                            self.new_word_definition[place] = ForthToken::Branch(BranchInfo::new(
                                "if".to_string(),
                                idx - place,
                                true,
                            ));
                        }
                        branch_stack.push(("else", idx));
                    }
                    "then" => {
                        // pop stack, insert new Unconditional Branch token with offset
                        if let Some((word, place)) = branch_stack.pop() {
                            self.new_word_definition[place] = ForthToken::Branch(BranchInfo::new(
                                word.to_string(),
                                idx - place,
                                true,
                            ));
                        }
                    }
                    "do" => {
                        // push onto branch_stack
                        branch_stack.push(("do", idx));
                    }
                    "loop" => {
                        // pop branch_stack, and set delta into loop as a negative distance, to jump back
                        if let Some((word, place)) = branch_stack.pop() {
                            self.new_word_definition[place] = ForthToken::Branch(BranchInfo::new(
                                word.to_string(),
                                idx - place,
                                true,
                            ));
                            // need to put in a value for move direction
                        }
                    }
                    _ => (),
                }
            }
            idx += 1;
        }
    }

    fn execute_token(&mut self, mut program_counter: usize, mut jumped: bool) -> (usize, bool) {
        // Execute a defined token
        self.step(); // gets a debug char if enabled
        program_counter += 1; // base assumption is we're processing one word
        match &self.token {
            ForthToken::Empty => return (program_counter, false),
            ForthToken::Integer(num) => {
                self.stack.push(*num);
            }
            ForthToken::Float(_num) => {
                // TBD: a separate stack is used for floating point calculations
            }
            ForthToken::Forward(info) => {
                // need a better way to capture forward and branch types' unique behaviors
                match info.word.as_str() {
                    "(" => {} // ignore comments
                    ".\"" => {
                        let tail = &info.tail[1..info.tail.len() - 1];
                        println!("{}", tail);
                    }
                    "s\"" => {
                        let txt = &info.tail;
                        self.text = info.tail[1..txt.len() - 1].to_owned();
                    }
                    "variable" => {
                        let index = self.variable_stack.len();
                        self.variable_stack.push(0); // create the location for the new variable
                        self.defined_variables
                            .insert(info.tail.trim().to_owned(), index as i64);
                        self.msg.warning(
                            "execute_token",
                            "Dealing with a variable called",
                            info.tail.clone(),
                        );
                    }
                    "see" => {
                        // ( "word name" -- ) print a word's definition or
                        // a builtin's documentation string
                        self.word_see(info.tail.trim());
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
                    "do" => {
                        // ( limit first -- )
                        // first time, (branch_flag == true) grab limit and first values and put them on the control stack
                        // if limit == current, jump over LOOP, otherwise increment current,
                        // set branch_flag false, and continue
                    }
                    "loop" => {
                        // unconditionally jump back to the matching loop
                    }
                    _ => (),
                }
            }
            ForthToken::Operator(op) => {
                macro_rules! pop2_push1 {
                    // Helper macro
                    ($word:expr, $expression:expr) => {
                        if let Some((j, k)) = self.pop_two(&$word) {
                            self.stack.push($expression(k, j));
                        }
                    };
                }
                macro_rules! pop1_push1 {
                    // Helper macro
                    ($word:expr, $expression:expr) => {
                        if let Some(x) = self.pop_one(&$word) {
                            self.stack.push($expression(x));
                        }
                    };
                }
                macro_rules! pop1 {
                    ($word:expr, $code:expr) => {
                        if let Some(x) = self.pop_one(&$word) {
                            $code(x);
                        }
                    };
                }
                match op.as_str() {
                    "+" => pop2_push1!("+", |a, b| a + b),
                    "-" => pop2_push1!("-", |a, b| a - b),
                    "*" => pop2_push1!("*", |a, b| a * b),
                    "/" => pop2_push1!("/", |a, b| a / b),
                    "mod" => pop2_push1!("mod", |a, b| a % b),
                    "<" => pop2_push1!("<", |a, b| if a < b { -1 } else { 0 }),
                    "." => pop1!(".", |a| print!("{a}")),
                    ".." => pop1!("..", |a| print!("{a}")),
                    "true" => self.stack.push(-1),
                    "false" => self.stack.push(0),
                    "=" => pop2_push1!("=", |a, b| if a == b { -1 } else { 0 }),
                    "0=" => pop1_push1!("0=", |a| if a == 0 { -1 } else { 0 }),
                    "0<" => pop1_push1!("0<", |a| if a < 0 { -1 } else { 0 }),
                    ".s" => {
                        // print stack contents
                        println!("{:?}", self.stack);
                    }
                    "show-stack" => {
                        self.show_stack = true;
                    }
                    "hide-stack" => {
                        self.show_stack = false;
                    }
                    ".s\"" => {
                        // print the saved string
                        println!("{:?}", self.text);
                    }
                    "emit" => {
                        if !self.stack_underflow("echo", 1) {
                            let n = self.stack.pop();
                            if let Some(n) = n {
                                if (0x20..=0x7f).contains(&n) {
                                    let c = n as u8 as char;
                                    print!("{}", c);
                                } else {
                                    self.msg.error("EMIT", "Arg out of range", n);
                                }
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
                            self.msg.warning("DUP", "Error - DUP: Stack is empty.", "");
                        }
                    }
                    "drop" => {
                        if !self.stack.is_empty() {
                            self.stack.pop();
                        } else {
                            self.msg.warning("DROP", "Stack is empty.", "");
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
                            self.msg.warning("SWAP", "Too few elements on stack.", "");
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
                                    self.msg.error("@", "Bad variable address", adr);
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
                        self.msg.warning("ABORT", "Terminating execution", "");
                        self.stack.clear();
                        self.parser.clear();
                        self.abort_flag = true;
                    }
                    "words" => {
                        for (key, _) in self.defined_words.iter() {
                            print!("{key} ");
                        }
                        println!();
                    }
                    "seeall" => {
                        for (key, _definition) in self.defined_words.iter() {
                            self.word_see(key);
                        }
                        for (key, index) in self.defined_variables.iter() {
                            self.variable_see(key, *index);
                        }
                    }
                    "stack-depth" => {
                        self.stack.push(self.stack.len() as i64);
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
                        Some(0) => self.msg.set_level(DebugLevel::Error),
                        Some(1) => self.msg.set_level(DebugLevel::Warning),
                        Some(2) => self.msg.set_level(DebugLevel::Info),
                        _ => self.msg.set_level(DebugLevel::Debug),
                    },
                    "debuglevel?" => {
                        println!("DebugLevel is {:?}", self.msg.get_level());
                    }
                    ":" => {
                        // Enter compile mode
                        self.set_compile_mode(true);
                    }
                    "step-on" => self.step_mode = true,
                    "step-off" => self.step_mode = false,
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
        (program_counter, jumped)
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
                    self.msg
                        .error("execute_definition", "Undefined word", word_name);
                    return;
                }
            }
            _ => {
                self.msg.error("execute_definition", "Definition error", "");
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
                                self.msg.debug("loaded", "processed", &self.token);
                            } else {
                                self.msg.debug("loaded", "No more tokens to read", "");
                                break;
                            }
                        }
                        std::mem::swap(&mut self.parser.reader, &mut previous_reader);
                        true
                    }
                    None => {
                        self.abort_flag = true;
                        self.msg.error("loaded", "Failed to create new reader", "");
                        false
                    }
                }
            }
            Err(error) => {
                self.msg.warning("loaded", error.to_string().as_str(), "");
                self.abort_flag = true;
                false
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

    fn word_see(&self, name: &str) {
        // if it's a word:
        match self.defined_words.get(name) {
            Some(definition) => {
                print!(": {name} ");
                for word in definition {
                    match word {
                        ForthToken::Integer(num) => print!("{num} "),
                        ForthToken::Float(num) => print!("f{num} "),
                        ForthToken::Operator(op) => print!("{op} "),
                        ForthToken::Branch(info) => {
                            print!("{}:{}:{} ", info.word, info.offset, info.branch_flag);
                        }
                        ForthToken::Forward(info) => {
                            print!("{}{} ", info.word, info.tail);
                        }
                        ForthToken::Empty => print!("ForthToken::Empty "),
                    }
                }
                println!(";");
            }
            None => {
                // check to see if it's a built-in
                let doc_string = self.builtin_doc.get(name);
                match doc_string {
                    Some(doc_string) => {
                        println!("Builtin: {name} {doc_string}");
                    }
                    None => self.msg.warning("SEE", "Word not found", name),
                }
            }
        }
    }

    fn get_stack(&self) -> String {
        if self.show_stack {
            format!("{:?}", self.stack)
        } else {
            "".to_owned()
        }
    }

    fn print_stack(&self) {
        println!("{}", self.get_stack());
    }

    fn print_variables(&self) {
        println!("Variables:");
        for (name, val) in self.defined_variables.iter() {
            println!("{name} = {val}");
        }
    }

    fn step(&mut self) {
        // controls step / debug functions
        if self.step_mode {
            match &self.token {
                ForthToken::Integer(num) => print!("{num}: Step> "),
                ForthToken::Float(num) => print!("f{num}: Step> "),
                ForthToken::Operator(op) => print!("{op}: Step> "),
                ForthToken::Branch(info) => {
                    print!("{}:{}:{}: Step> ", info.word, info.offset, info.branch_flag);
                }
                ForthToken::Forward(info) => {
                    print!("{}{}: Step> ", info.word, info.tail);
                }
                ForthToken::Empty => print!("ForthToken::Empty: Step> "),
            }
            io::stdout().flush().unwrap();
            match self.parser.reader.read_char() {
                Some('s') => self.print_stack(),
                Some('v') => self.print_variables(),
                Some('a') => {
                    self.print_stack();
                    self.print_variables();
                }
                Some('c') => self.step_mode = false,
                Some(_) | None => {}
            }
        }
    }
}
