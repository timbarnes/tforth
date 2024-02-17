//The tForth interpreter struct and implementation

mod builtin;

use std::collections::HashMap;
use std::io::{self, Write};

use crate::doc;
use crate::messages::Msg;
use crate::reader::Reader;
use crate::tokenizer::{ForthToken, Tokenizer};
use builtin::BuiltInFn;

#[derive(Debug)]
struct ControlFrame {
    id: usize,
    incr: i64,
    end: i64,
}

impl ControlFrame {
    fn new(id: usize, start: i64, end: i64) -> ControlFrame {
        ControlFrame {
            id,
            incr: start,
            end,
        }
    }
}
#[derive(Clone, Debug)]
pub enum OpCode {
    // used in compiled definitions to reference objects
    B(usize),     // builtin
    Jif(usize),   // if (branch)
    Jelse(usize), // else (branch)
    Jthen(usize), // then (branch)
    Jfor(usize),  // for (branch)
    Jnext(usize), // next (branch)
    W(usize),     // defined word
    V(usize),     // variable
    C(usize),     // constant
    L(i64),       // literal
    F(f64),       // float literal
    S(String),    // an inline string
    Noop,         // do nothing
}

//#[derive(Debug)]
pub struct TF {
    pub stack: Vec<i64>,             // the numeric stack, currently integers
    pub dictionary: Vec<ForthToken>, // the dictionary: keys (words) and their definitions
    pub builtins: Vec<BuiltInFn>,    // the dictionary of builtins
    pub variable_stack: Vec<i64>,    // where variables are stored
    pub defined_variables: HashMap<String, i64>, // separate hashmap for variables
    pub defined_constants: HashMap<String, i64>, // separate hashmap for constants
    return_stack: Vec<i64>,          // for do loops etc.
    builtin_doc: HashMap<String, String>, // doc strings for built-in words
    text: String,                    // the current s".."" string
    file_mode: FileMode,
    compile_mode: bool, // true if compiling a word
    abort_flag: bool,   // true if abort has been called
    exit_flag: bool,    // set when the "bye" word is executed.
    pub msg: Msg,
    parser: Tokenizer,
    new_word_name: String,
    new_word_definition: Vec<OpCode>,
    token_ptr: (usize, ForthToken),
    show_stack: bool, // show the stack at the completion of a line of interaction
    step_mode: bool,
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
        let doc_strings = doc::build_doc_strings();
        if let Some(reader) = Reader::new(None, main_prompt, multiline_prompt, Msg::new()) {
            let parser = Tokenizer::new(reader);
            TF {
                stack: Vec::new(),
                dictionary: Vec::new(),
                builtins: Vec::new(),
                text: String::new(),
                variable_stack: Vec::new(),
                // constant_stack: Vec::new(),
                defined_variables: HashMap::new(),
                defined_constants: HashMap::new(),
                return_stack: Vec::new(),
                builtin_doc: doc_strings,
                file_mode: FileMode::Unset,
                compile_mode: false,
                abort_flag: false,
                exit_flag: false,
                msg: Msg::new(),
                parser,
                new_word_name: String::new(),
                new_word_definition: Vec::new(),
                token_ptr: (0, ForthToken::Empty),
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

    fn stack_underflow(&self, op: &str, n: usize) -> bool {
        if self.stack.len() < n {
            self.msg.error(op, "Stack underflow", None::<bool>);
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
                self.msg.error(word, "Stack underflow", None::<bool>);
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
                if self.compile_mode {
                    self.compile_token();
                } else {
                    // we're in immediate mode
                    self.execute_token(0, false);
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
                    self.compile_mode = false;
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
            ForthToken::Forward(info) => op_code = OpCode::S(info.tail.clone()),
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
                OpCode::Jnext(delta) => {
                    if let Some(slot) = loop_stack.pop() {
                        self.new_word_definition[slot] = OpCode::Jfor(step - slot - 1);
                    }
                    self.new_word_definition[step] = OpCode::Jnext(step - delta + 1);
                }
                _ => {}
            }
            step += 1;
        }
    }

    fn execute_token(&mut self, mut program_counter: usize, jumped: bool) -> (usize, bool) {
        // Execute a defined token
        self.step(); // gets a debug char if enabled
        program_counter += 1; // base assumption is we're processing one word
        match &self.token_ptr.1 {
            ForthToken::Empty => return (program_counter, false),
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
                        let tail = &info.tail[1..info.tail.len() - 1];
                        print!("{}", tail);
                    }
                    "s\"" => {
                        let txt = &info.tail;
                        self.text = info.tail[1..txt.len() - 1].to_owned();
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
                        let result = self.find_definition(info.tail.trim()); // gets the index of a word
                        match result {
                            Some(idx) => self.word_see(idx),
                            None => {
                                self.msg
                                    .warning("see", "word not found", Some(info.tail.trim()))
                            }
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
            _ => {}
        }
        (program_counter, jumped)
    }

    fn execute_definition(&mut self) {
        // execute a word defined in forth
        // see if the word is in the dictionary.
        // if so, iterate over the definition, using execute_token()
        let mut program_counter = 0;
        let mut jumped = false;
        match &self.token_ptr.1 {
            ForthToken::Definition(word_name, _) => {
                match self.find_definition(word_name) {
                    Some(index) => {
                        let definition = &self.dictionary[index].clone();
                        match definition {
                            ForthToken::Definition(_n, code) => {
                                while program_counter < code.len() {
                                    if self.abort_flag {
                                        // code.clear();
                                        self.stack.clear();
                                        self.return_stack.clear();
                                        self.abort_flag = false;
                                        break;
                                    } else {
                                        (program_counter, jumped) = self.execute_opcode(
                                            &code[program_counter],
                                            program_counter,
                                            jumped,
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    None => {
                        if self.defined_constants.contains_key(word_name) {
                            self.stack.push(self.defined_constants[word_name]); // push the index on the stack
                        } else {
                            self.msg
                                .error("execute_definition", "Undefined word", Some(word_name));
                            return;
                        }
                    }
                };
            }
            _ => {
                self.msg
                    .error("execute_definition", "Definition error", None::<bool>);
            }
        }
    }

    fn execute_opcode(
        &mut self,
        op_code: &OpCode,
        mut program_counter: usize,
        mut jumped: bool,
    ) -> (usize, bool) {
        // run a single opcode, updating the PC if required
        match op_code {
            OpCode::L(n) => self.stack.push(*n),
            OpCode::S(st) => print!("{}", st),
            OpCode::B(code) => self.execute_builtin(*code),
            OpCode::V(idx) => self.stack.push(*idx as i64), // f_variable returns the address of the variable
            OpCode::C(idx) => self.stack.push(self.f_constant(*idx)), // get the constant's value
            OpCode::Jif(offset) => {
                if !self.stack_underflow("if", 1) {
                    let b = self.stack.pop();
                    if b.unwrap() == 0 {
                        program_counter += offset;
                        jumped = true;
                    } else {
                        jumped = false;
                    }
                }
            } // conditional semantics
            OpCode::Jelse(offset) => {
                if jumped {
                    jumped = false
                } else {
                    program_counter += offset;
                    jumped = true;
                }
            }
            OpCode::Jthen(_offset) => {
                jumped = false;
            } // loop semantics
            OpCode::Jfor(offset) => {
                let count = self.stack.pop();
                match count {
                    Some(count) => {
                        let counter = count - 1;
                        if counter <= 0 {
                            program_counter += offset;
                            jumped = true;
                        } else {
                            self.return_stack.push(counter);
                            jumped = false;
                        }
                    }
                    None => {} // stack error
                }
            }
            OpCode::Jnext(offset) => {
                let count = self.return_stack.pop();
                match count {
                    Some(count) => {
                        self.stack.push(count);
                        program_counter -= offset;
                        jumped = true;
                    }
                    None => {}
                }
            }
            _ => {}
        }
        (program_counter + 1, jumped)
    }

    fn execute_builtin(&mut self, code: usize) {
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
                        self.abort_flag = true;
                        self.msg
                            .error("loaded", "Failed to create new reader", None::<bool>);
                        false
                    }
                }
            }
            Err(error) => {
                self.msg
                    .warning("loaded", error.to_string().as_str(), None::<bool>);
                self.abort_flag = true;
                false
            }
        }
    }

    fn loaded(&mut self) {
        // Load a file of forth code. Initial implementation is not intended to be recursive.
        // attempt to open the file, return an error if not possible
        self.load_file(&self.text.clone());
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
                ForthToken::Definition(n, _) | ForthToken::Variable(n, _) => {
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

    fn find(&self, name: &str) -> Option<usize> {
        // find a word if it's defined; search from the newest to the oldest
        if self.dictionary.len() > 0 {
            for (i, token) in self.dictionary.iter().rev().enumerate() {
                match token {
                    ForthToken::Definition(n, _)
                    | ForthToken::Variable(n, _)
                    | ForthToken::Constant(n, _) => {
                        if n.as_str() == name {
                            return Some(self.dictionary.len() - i - 1);
                        }
                    }
                    _ => {} // should only be definitions, variables and constants in the list
                }
            }
        }
        None
    }

    fn variable_see(&self, name: &str, index: i64) {
        let idx = index.max(0) as usize;
        let value = self.variable_stack[idx];
        println!("Variable {name}: {value}");
    }

    fn word_see(&self, index: usize) {
        // soon adding variables and constants
        let token = &self.dictionary[index];
        match token {
            ForthToken::Definition(name, def) => {
                print!(": {name} ");
                for word in def {
                    match word {
                        OpCode::F(f) => print!("f{} ", f),
                        OpCode::B(idx) => print!("B{idx} "),
                        OpCode::W(_idx) => print!("{name} "),
                        OpCode::Jif(offset) => print!("if:{offset} "),
                        OpCode::Jelse(offset) => print!("else:{offset} "),
                        OpCode::Jthen(offset) => print!("then:{offset} "),
                        OpCode::Jfor(offset) => print!("for:{offset} "),
                        OpCode::Jnext(offset) => print!("next:{offset} "),
                        OpCode::S(info) => {
                            print!(".\" {info} ");
                        }
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
                    }
                }
                println!(";");
            }
            ForthToken::Variable(name, val) => print!("V {name}={val} "),
            ForthToken::Constant(name, val) => print!("C {name}={val} "),
            _ => {}
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
}
