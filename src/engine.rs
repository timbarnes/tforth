//The tForth interpreter struct and implementation
use std::collections::HashMap;
use std::io::{self, Write};

use crate::internals::builtin::BuiltInFn;
// use crate::internals::compiler::*;
use crate::messages::Msg;
use crate::reader::Reader;
use crate::tokenizer::{ForthToken, Tokenizer};

pub const DATA_SIZE: usize = 10000;
pub const STRING_SIZE: usize = 5000;
pub const BUF_SIZE: usize = 132;
pub const TIB_START: usize = 0;
pub const PAD_START: usize = TIB_START + BUF_SIZE;
pub const STR_START: usize = PAD_START + BUF_SIZE;
pub const ALLOC_START: usize = PAD_START + BUF_SIZE;
pub const STACK_START: usize = DATA_SIZE / 2; // stack counts up
pub const RET_START: usize = STACK_START - 1; // return stack counts downwards
pub const WORD_START: usize = 0; // data area counts up from the bottom (builtins, words, variables etc.)
pub const TRUE: i64 = -1; // forth convention for true and false
pub const FALSE: i64 = 0;
pub const ADDRESS_MASK: usize = 0x00FFFFFFFFFFFFFF; // to get rid of the immediate flag
pub const IMMEDIATE_MASK: usize = 0x4000000000000000; // the immediate bit

// Indices into builtins to drive execution of each data type
pub const BUILTIN: i64 = 0;
pub const VARIABLE: i64 = 1;
pub const CONSTANT: i64 = 2;
pub const LITERAL: i64 = 3;
pub const STRING: i64 = 4;
pub const DEFINITION: i64 = 5;

#[derive(Clone, Debug)]
pub enum OpCode {
    // used in compiled definitions to reference objects
    B(usize),        // builtin
    D(String),       // a definition's header
    Lparen(String),  // paren (comment)
    Jif(usize),      // if (branch)
    Jelse(usize),    // else (branch)
    Jthen(usize),    // then (branch)
    Jfor(usize),     // for (branch)
    Jnext(usize),    // next (branch)
    W(usize),        // defined word
    V(usize),        // variable reference
    C(usize),        // constant reference
    L(i64),          // literal
    F(f64),          // float literal
    Lstring(String), // an inline string
    Noop,            // do nothing
}

//#[derive(Debug)]
pub struct TF {
    pub data: [i64; DATA_SIZE],
    pub strings: [char; STRING_SIZE], // storage for strings
    pub stack: Vec<i64>,              // the numeric stack, currently integers
    pub dictionary: Vec<ForthToken>,  // the dictionary: keys (words) and their definitions
    pub builtins: Vec<BuiltInFn>,     // the dictionary of builtins
    pub defined_variables: HashMap<String, i64>, // separate hashmap for variables
    pub defined_constants: HashMap<String, i64>, // separate hashmap for constants
    pub return_stack: Vec<i64>,       // for do loops etc.
    pub here_ptr: usize,
    pub stack_ptr: usize, // top of the linear space stack
    pub context_ptr: usize,
    pub eval_ptr: usize, // used to turn compile mode on and off
    pub base_ptr: usize,
    pub pad_ptr: usize,    // the current s".."" string
    pub string_ptr: usize, // points to the beginning of free string space
    pub last_ptr: usize,   // points to name of top word
    pub hld_ptr: usize,    // for numeric string work
    pub file_mode: FileMode,
    pub compile_ptr: usize, // true if compiling a word
    pub pc_ptr: usize,      // program counter
    pub abort_ptr: usize,   // true if abort has been called
    pub tib_ptr: usize,     // TIB
    pub tib_size_ptr: usize,
    pub tib_in_ptr: usize,
    exit_flag: bool, // set when the "bye" word is executed.
    pub msg: Msg,
    pub parser: Tokenizer,
    new_word_name: String,
    new_word_definition: Vec<OpCode>,
    token_ptr: (usize, ForthToken),
    pub show_stack: bool, // show the stack at the completion of a line of interaction
    pub step_mode: bool,
}

#[derive(Debug)]
pub enum FileMode {
    // used for file I/O
    ReadWrite,
    ReadOnly,
    Unset,
}

impl TF {
    // ForthInterpreter struct implementations
    pub fn new(main_prompt: &str, multiline_prompt: &str) -> TF {
        if let Some(reader) = Reader::new(None, main_prompt, multiline_prompt, Msg::new()) {
            let parser = Tokenizer::new(reader);
            TF {
                data: [0; DATA_SIZE],
                strings: [' '; STRING_SIZE],
                stack: Vec::new(),
                dictionary: Vec::new(),
                builtins: Vec::new(),
                defined_variables: HashMap::new(),
                defined_constants: HashMap::new(),
                return_stack: Vec::new(),
                here_ptr: WORD_START,
                stack_ptr: STACK_START,
                string_ptr: 0,
                context_ptr: 0,
                eval_ptr: 0,
                base_ptr: 0,
                pad_ptr: 0,
                last_ptr: 0,
                hld_ptr: 0,
                file_mode: FileMode::Unset,
                compile_ptr: 0,
                pc_ptr: 0,
                abort_ptr: 0,
                tib_ptr: 0,
                tib_size_ptr: 0,
                tib_in_ptr: 0,
                exit_flag: false,
                msg: Msg::new(),
                parser,
                new_word_name: String::new(),
                new_word_definition: Vec::new(),
                token_ptr: (0, ForthToken::Empty),
                show_stack: true,
                step_mode: false,
            }
        } else {
            panic!("unable to create reader");
        }
    }

    pub fn cold_start(&mut self) {
        self.u_insert_variables();
        //self.f_insert_builtins();
        self.add_builtins();
        self.u_insert_code();
        self.add_variables();
    }

    pub fn get_compile_mode(&mut self) -> bool {
        if self.get_var(self.compile_ptr) == 0 {
            false
        } else {
            true
        }
    }
    pub fn set_compile_mode(&mut self, value: bool) {
        self.set_var(self.compile_ptr, if value { -1 } else { 0 });
    }

    pub fn set_abort_flag(&mut self, v: bool) {
        self.set_var(self.abort_ptr, if v { -1 } else { 0 });
    }
    pub fn get_abort_flag(&mut self) -> bool {
        if self.get_var(self.abort_ptr) == 0 {
            false
        } else {
            true
        }
    }

    pub fn set_program_counter(&mut self, val: usize) {
        self.set_var(self.pc_ptr, val as i64);
    }
    fn get_program_counter(&mut self) -> usize {
        self.get_var(self.pc_ptr) as usize
    }
    fn increment_program_counter(&mut self, val: usize) {
        let new = self.get_program_counter() + val;
        self.set_var(self.pc_ptr, (new) as i64);
    }
    fn decrement_program_counter(&mut self, val: usize) {
        let new = self.get_program_counter() - val;
        self.set_var(self.pc_ptr, (new) as i64);
    }

    pub fn set_exit_flag(&mut self) {
        // Method executed by "bye"
        self.exit_flag = true;
    }

    pub fn should_exit(&self) -> bool {
        // Method to determine if we should exit
        self.exit_flag
    }

    pub fn stack_underflow(&self, op: &str, n: usize) -> bool {
        if self.stack.len() < n {
            self.msg.error(op, "Stack underflow", None::<bool>);
            true
        } else {
            false
        }
    }

    pub fn pop_one(&mut self, word: &str) -> Option<i64> {
        match self.stack.pop() {
            Some(value) => Some(value),
            None => {
                self.msg.error(word, "Stack underflow", None::<bool>);
                None
            }
        }
    }

    pub fn pop_two(&mut self, word: &str) -> Option<(i64, i64)> {
        let (val1, val2) = (self.stack.pop(), self.stack.pop());
        match val1 {
            Some(value1) => match val2 {
                Some(value2) => Some((value1, value2)),
                None => None,
            },
            None => {
                self.msg.error(word, "Stack underflow", None::<bool>);
                None
            }
        }
    }

    pub fn process_token(&mut self) -> bool {
        let opt_token = self.parser.get_token(&self.get_stack()); // Prompt if necessary, return a token
        match opt_token {
            Some(token) => {
                self.msg
                    .debug("execute_token", "operator is", Some(&self.token_ptr));
                self.token_ptr = (0, token);
                match self.token_ptr.1.clone() {
                    ForthToken::Empty => {
                        return true; // An empty line
                    }
                    ForthToken::Operator(name) => {
                        // Builtin, definition, variable or constant (or undefined)
                        // check builtins first
                        let builtin = self.find_builtin(&name);
                        match builtin {
                            Some((index, _tok)) => {
                                // it's a builtin
                                self.token_ptr =
                                    (index, ForthToken::Builtin(name.to_owned(), index));
                            }
                            None => {
                                let def = self.find_definition(&name);
                                match def {
                                    Some(idx) => {
                                        // it's in the dictionary
                                        self.token_ptr = (idx, self.dictionary[idx].clone());
                                    }
                                    None => {} // it's something special, or undefined
                                }
                            }
                        }
                    }
                    _ => {}
                }
                // Any valid token
                if self.get_compile_mode() {
                    self.compile_token();
                } else {
                    // we're in immediate mode
                    self.execute_token();
                }
                true
            }
            None => {
                false // Signals end of file
            }
        }
    }

    fn compile_token(&mut self) {
        // We're in compile mode: creating a new definition
        let (idx, tok) = &self.token_ptr; // the word being compiled
        let mut op_code = OpCode::Noop;
        match tok {
            ForthToken::Operator(tstring) | ForthToken::Definition(tstring, _) => {
                if tstring == ";" {
                    // we are at the end of the definition
                    self.calculate_branches();
                    self.dictionary.push(ForthToken::Definition(
                        self.new_word_name.clone(),
                        self.new_word_definition.clone(),
                    ));
                    self.new_word_name.clear();
                    self.new_word_definition.clear();
                    self.set_compile_mode(false);
                } else if self.new_word_name.is_empty() {
                    // We've found the word name
                    self.new_word_name = tstring.to_string();
                } else {
                    // build the opcode and push it onto the definition
                    op_code = OpCode::W(*idx);
                }
            }
            ForthToken::Integer(val) => op_code = OpCode::L(*val),
            ForthToken::Builtin(_n, code) => op_code = OpCode::B(*code),
            ForthToken::Jump(name, delta) => {
                match name.as_str() {
                    "if" => op_code = OpCode::Jif(*delta),
                    "else" => op_code = OpCode::Jelse(*delta),
                    "then" => op_code = OpCode::Jthen(*delta),
                    "for" => op_code = OpCode::Jfor(*delta),
                    "next" => op_code = OpCode::Jnext(*delta),
                    _ => {}
                };
            }
            ForthToken::Variable(_, _) => op_code = OpCode::V(*idx),
            ForthToken::Constant(_, _) => op_code = OpCode::C(*idx),
            ForthToken::Forward(info) => match info.word.as_str() {
                ".\"" => op_code = OpCode::Lstring(info.tail.to_owned()),
                "(" => op_code = OpCode::Lparen(info.tail.to_owned()),
                _ => op_code = OpCode::Noop,
            },
            _ => op_code = OpCode::Noop,
        }
        match op_code {
            OpCode::Noop => {}
            _ => self.new_word_definition.push(op_code),
        }
    }

    fn calculate_branches(&mut self) {
        // replace words that incorporate branches with OpCode::B
        // and set up offsets as required.
        let mut loop_stack = Vec::<usize>::new();
        let mut conditional_stack = Vec::<usize>::new();
        let mut step = 0; // points to the current token
        while step < self.new_word_definition.len() {
            match self.new_word_definition[step] {
                OpCode::Jif(_delta) => {
                    conditional_stack.push(step); // the location of the if
                }
                OpCode::Jelse(_delta) => {
                    // pop stack, insert updated Jif
                    if let Some(slot) = conditional_stack.pop() {
                        self.new_word_definition[slot] = OpCode::Jif(step - slot);
                        conditional_stack.push(step);
                    }
                }
                OpCode::Jthen(_delta) => {
                    // pop stack, insert new if or else token with offset
                    if let Some(slot) = conditional_stack.pop() {
                        match &self.new_word_definition[slot] {
                            OpCode::Jif(_delta) => {
                                self.new_word_definition[slot] = OpCode::Jif(step - slot);
                            }
                            OpCode::Jelse(_delta) => {
                                self.new_word_definition[slot] = OpCode::Jelse(step - slot);
                            }
                            _ => {}
                        }
                    }
                }
                OpCode::Jfor(_delta) => {
                    // push onto branch_stack
                    loop_stack.push(step);
                }
                OpCode::Jnext(_delta) => {
                    if let Some(slot) = loop_stack.pop() {
                        self.new_word_definition[slot] = OpCode::Jfor(step - slot);
                        self.new_word_definition[step] = OpCode::Jnext(step - slot + 1);
                    }
                }
                _ => {}
            }
            step += 1;
        }
    }

    fn execute_token(&mut self) {
        // Execute a defined token
        self.step(); // gets a debug char if enabled
        match &self.token_ptr.1 {
            ForthToken::Empty => return,
            ForthToken::Integer(num) => {
                self.stack.push(*num);
            }
            ForthToken::Float(_num) => {
                // TBD: a separate stack is used for floating point calculations
            }
            ForthToken::Forward(info) => {
                match info.word.as_str() {
                    "(" => {} // ignore comments
                    ".\"" => {
                        let tail = info.tail[1..info.tail.len() - 1].to_string();
                        print!("{}", tail);
                    }
                    "s\"" => {
                        let tail: String = info.tail[1..info.tail.len() - 1].to_owned();
                        self.set_string(self.pad_ptr, tail.as_str());
                    }
                    "variable" => {
                        // add it to the dictionary
                        let var = ForthToken::Variable(info.tail.trim().to_owned(), 0);
                        self.dictionary.push(var);
                        /* let index = self.variable_stack.len();
                        self.variable_stack.push(0); // create the location for the new variable
                        self.defined_variables
                            .insert(info.tail.trim().to_owned(), index as i64); */
                        self.msg.debug(
                            "execute_token",
                            "Dealing with a variable called",
                            Some(&info.tail),
                        );
                    }
                    "constant" => {
                        // Create the element and store its value from the stack
                        if let Some(constant_value) = self.stack.pop() {
                            self.defined_constants
                                .insert(info.tail.trim().to_owned(), constant_value);
                            self.msg.debug(
                                "execute_token",
                                "Dealing with a constant called",
                                Some(&info.tail),
                            );
                        } else {
                            self.msg.error(
                                "constant",
                                "Stack underflow.",
                                Some("Constant needs value"),
                            );
                        }
                    }
                    "see" => {
                        // ( "word name" -- ) print a word's definition or
                        // a builtin's documentation string
                        let result = self.find(info.tail.trim()); // gets the index of a word
                        match result {
                            Some(idx) => self.see_word(idx),
                            None => {}
                        }
                    }
                    _ => {}
                }
            }
            ForthToken::Definition(_name, _def) => self.execute_definition(),
            ForthToken::Builtin(_name, code) => self.execute_builtin(*code),
            ForthToken::Variable(_name, _val) => {
                self.stack.push(self.token_ptr.0 as i64);
            }
            ForthToken::StringVar(_name, _val) => {
                self.stack.push(self.token_ptr.0 as i64);
            }
            _ => {}
        }
    }

    fn execute_definition(&mut self) {
        // execute a word defined in forth
        // see if the word is in the dictionary.
        // if so, iterate over the definition, using execute_opcode()
        // save the value
        match &self.token_ptr.1 {
            ForthToken::Definition(word_name, _) => match self.find_definition(word_name) {
                Some(index) => self.execute_word(index),
                None => self
                    .msg
                    .error("execute_definition", "Undefined word", Some(word_name)),
            },
            _ => {
                self.msg
                    .error("execute_definition", "Definition error", None::<bool>);
                self.set_abort_flag(true);
            }
        }
    }

    pub fn execute_word(&mut self, index: usize) {
        // executes the code part of a word at index
        if let ForthToken::Definition(_, code) = self.dictionary[index].clone() {
            let pc = self.get_program_counter() as i64;
            self.return_stack.push(pc as i64);
            self.set_program_counter(0);
            // loop through the definition
            while self.get_program_counter() < code.len() {
                if self.get_abort_flag() {
                    self.f_quit();
                    return;
                } else {
                    let opcode = &code[self.get_program_counter()];
                    self.execute_opcode(opcode);
                }
            }
            // pop the program counter and restore it
            if let Some(pc) = self.return_stack.pop() {
                self.set_program_counter(pc as usize);
            } else {
                self.msg
                    .error("execute-definition", "Return stack underflow", None::<bool>);
            }
        }
    }

    pub fn execute_opcode(&mut self, op_code: &OpCode) {
        // run a single opcode, updating the PC if required
        self.increment_program_counter(1);
        match op_code {
            OpCode::L(n) => self.stack.push(*n),
            OpCode::Lstring(st) => print!("{}", st),
            OpCode::B(code) => self.execute_builtin(*code),
            OpCode::D(_name) => {} // reserved
            OpCode::W(idx) => self.execute_word(*idx),
            OpCode::V(idx) => self.stack.push(*idx as i64), // f_variable returns the address of the variable
            OpCode::C(idx) => self.stack.push(self.f_constant(*idx)), // get the constant's value
            OpCode::Jif(offset) => {
                if !self.stack_underflow("if", 1) {
                    let b = self.stack.pop();
                    if b.unwrap() == 0 {
                        self.increment_program_counter(*offset);
                    }
                }
            }
            OpCode::Jelse(offset) => {
                self.increment_program_counter(*offset);
            }
            OpCode::Jthen(_offset) => {}
            OpCode::Jfor(offset) => {
                let count = self.stack.pop();
                match count {
                    Some(counter) => {
                        self.return_stack.push(counter);
                        if counter < 0 {
                            self.increment_program_counter(*offset - 1);
                        }
                    }
                    None => {} // stack error
                }
            }
            OpCode::Jnext(offset) => {
                let count = self.return_stack.pop();
                match count {
                    Some(count) => {
                        if count > 0 {
                            self.stack.push(count - 1);
                            self.decrement_program_counter(*offset);
                        }
                    }
                    None => self
                        .msg
                        .error("NEXT", "Return stack underflow", None::<bool>),
                }
            }
            _ => {}
        }
    }

    pub fn execute_builtin(&mut self, code: usize) {
        let op = &self.builtins[code];
        let func = op.code;
        func(self);
    }

    pub fn load_file(&mut self, path: &String) -> bool {
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
                                self.msg.debug("loaded", "processed", Some(&self.token_ptr));
                            } else {
                                self.msg
                                    .debug("loaded", "No more tokens to read", None::<bool>);
                                break;
                            }
                        }
                        std::mem::swap(&mut self.parser.reader, &mut previous_reader);
                        true
                    }
                    None => {
                        self.set_abort_flag(true);
                        self.msg
                            .error("loaded", "Failed to create new reader", None::<bool>);
                        false
                    }
                }
            }
            Err(error) => {
                self.msg
                    .warning("loaded", error.to_string().as_str(), None::<bool>);
                self.set_abort_flag(true);
                false
            }
        }
    }

    pub fn loaded(&mut self) {
        // Load a file of forth code. Initial implementation is not intended to be recursive.
        // attempt to open the file, return an error if not possible
        let file_name = self.get_string(self.pad_ptr);
        self.load_file(&file_name);
    }

    fn f_constant(&self, idx: usize) -> i64 {
        match &self.dictionary[idx] {
            ForthToken::Constant(_n, v) => *v,
            _ => 0,
        }
    }

    fn find_definition(&self, name: &str) -> Option<usize> {
        for i in (0..self.dictionary.len()).rev() {
            match &self.dictionary[i] {
                ForthToken::Definition(n, _)
                | ForthToken::Variable(n, _)
                | ForthToken::StringVar(n, _)
                | ForthToken::Constant(n, _) => {
                    //println!("{}:{}", i, n);
                    if n == name {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn find_builtin(&self, name: &str) -> Option<(usize, &BuiltInFn)> {
        for i in 0..self.builtins.len() {
            if self.builtins[i].name == name {
                return Some((i, &self.builtins[i]));
            }
        }
        None
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        // find a word if it's defined; search from the newest to the oldest
        match self.find_definition(name) {
            Some(idx) => return Some(idx),
            None => {}
        }
        match self.find_builtin(name) {
            Some((idx, _)) => return Some(idx + 1000),
            None => {}
        }
        None
    }

    pub fn see_word(&self, index: usize) {
        // soon adding variables and constants
        if index < 1000 {
            // it's a definition
            match &self.dictionary[index] {
                ForthToken::Definition(name, def) => {
                    print!(": {name} ");
                    for word in def {
                        match word {
                            OpCode::F(f) => print!("f{} ", f),
                            OpCode::B(idx) => print!("{} ", &self.builtins[*idx].name),
                            OpCode::W(idx) => {
                                let token = &self.dictionary[*idx];
                                match token {
                                    ForthToken::Definition(name, _code) => {
                                        print!("{} ", name);
                                    }
                                    _ => {}
                                }
                            }
                            OpCode::Jif(offset) => print!("if:{offset} "),
                            OpCode::Jelse(offset) => print!("else:{offset} "),
                            OpCode::Jthen(offset) => print!("then:{offset} "),
                            OpCode::Jfor(offset) => print!("for:{offset} "),
                            OpCode::Jnext(offset) => print!("next:{offset} "),
                            OpCode::Lstring(info) => print!(".\" {info} "),
                            OpCode::Lparen(txt) => print!("({} ", txt),
                            OpCode::L(n) => print!("{n} "),
                            OpCode::V(idx) => {
                                if let ForthToken::Variable(name, val) = &self.dictionary[*idx] {
                                    print!("V {}={} ", name, val);
                                }
                            }
                            OpCode::C(idx) => {
                                if let ForthToken::Constant(name, val) = &self.dictionary[*idx] {
                                    print!("C {}={} ", name, val);
                                }
                            }
                            OpCode::Noop => print!("Noop "),
                            OpCode::D(_name) => print!("!!Definition not implemented"),
                        }
                    }
                    println!(";");
                }
                ForthToken::Variable(name, val) => println!("Variable {name} = {val} "),
                ForthToken::Constant(name, val) => println!("Constant {name} = {val} "),
                ForthToken::StringVar(name, val) => println!("String {name} = <{val}> "),
                _ => {}
            }
        } else {
            // it's a builtin
            match &self.builtins[index - 1000] {
                BuiltInFn { name, code, doc } => {
                    println!("BuiltIn {}: {}", index - 1000, doc);
                }
                _ => {}
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
        println!("Calculation Stack: {}", self.get_stack());
    }

    fn print_return_stack(&self) {
        println!("Return     stack: {:?}", self.return_stack);
    }

    fn step(&mut self) {
        // controls step / debug functions
        if self.step_mode {
            match &self.token_ptr.1 {
                ForthToken::Integer(num) => print!("{num}: Step> "),
                ForthToken::Float(num) => print!("f{num}: Step> "),
                ForthToken::Operator(op) => print!("{op}: Step> "),
                ForthToken::Jump(name, offset) => {
                    print!("{name}:{}: Step> ", offset);
                }
                ForthToken::Forward(info) => {
                    print!("{}{}: Step> ", info.word, info.tail);
                }
                ForthToken::Builtin(name, code) => print!("{}:{:?}", name, code),
                ForthToken::Definition(name, _def) => print!("{name} "),
                ForthToken::Empty => print!("ForthToken::Empty: Step> "),
                ForthToken::Variable(n, v) => print!("{}={}", n, v),
                _ => print!("variable or constant???"),
            }
            io::stdout().flush().unwrap();
            match self.parser.reader.read_char() {
                Some('s') => {
                    self.print_stack();
                    self.print_return_stack();
                }
                // Some('v') => self.print_variables(),
                Some('a') => {
                    self.print_stack();
                    //self.print_variables();
                }
                Some('c') => self.step_mode = false,
                Some(_) | None => {}
            }
        }
    }

    /* pub fn pack_string(&self, input: &str) -> Vec<usize> {
        // tries to pack a string
        let mut output = Vec::new();
        let mut tmp = input.len();
        println!("{:#x}", tmp);

        let words: usize = input.len() / 7 + 1;
        //println!("Words:{words}");
        let mut i = 0;
        for c in input.chars() {
            i += 1;
            if i % 8 == 0 {
                output.push(tmp);
                tmp = 0;
            }
            let shift = i % 8;
            let new = (c as u8 as usize) << 8 * shift;
            println!("{shift} {:#x}", new);
            tmp |= (c as u8 as usize) << 8 * shift;
            //println!("tmp{:#x}", tmp);
        }
        output.push(tmp);
        //println!("Finished packing");
        output
    } */
}
