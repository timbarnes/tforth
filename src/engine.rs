//The tForth interpreter struct and implementation

mod builtin;

use std::collections::HashMap;
use std::io::{self, Write};

use crate::doc;
use crate::messages::Msg;
use crate::reader::Reader;
use crate::tokenizer::{BranchInfo, ForthToken, Tokenizer};
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

//#[derive(Debug)]
pub struct TF {
    pub stack: Vec<i64>,             // the numeric stack, currently integers
    pub dictionary: Vec<ForthToken>, // the dictionary: keys (words) and their definitions
    pub builtins: Vec<BuiltInFn>,    // the dictionary of builtins
    pub variable_stack: Vec<i64>,    // where variables are stored
    pub defined_variables: HashMap<String, i64>, // separate hashmap for variables
    pub defined_constants: HashMap<String, i64>, // separate hashmap for constants
    control_stack: Vec<ControlFrame>, // for do loops etc.
    builtin_doc: HashMap<String, String>, // doc strings for built-in words
    text: String,                    // the current s".."" string
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
        let new_token = self.parser.get_token(&self.get_stack()); // Prompt if necessary, return a token
        match new_token {
            Some(new_token) => {
                self.msg
                    .debug("execute_token", "operator is", Some(&self.token));
                self.token = new_token;
                match &self.token {
                    ForthToken::Empty => {
                        return true; // An empty line
                    }
                    ForthToken::Operator(name) => {
                        // Figure out if it's a definition or not
                        let idx = self.find_definition(name.as_str());
                        match idx {
                            Some(_idx) => {
                                self.token = ForthToken::Definition(name.clone(), Vec::new())
                            }
                            None => {
                                let idx = self.find_builtin(name);
                                match idx {
                                    Some(index) => {
                                        self.token =
                                            ForthToken::Builtin(name.to_owned(), index as u8)
                                    }
                                    None => {}
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
        let tok = &self.token; // the word being compiled
        match tok {
            ForthToken::Operator(tstring) => {
                if tstring == ";" {
                    // we are at the end of the definition
                    self.calculate_branches();
                    println!(
                        "compile_token: new definition is : {} {:?} ;",
                        &self.new_word_name, &self.new_word_definition
                    );
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
                } else if tstring == ":" {
                    self.msg
                        .warning("compile_token", "Illegal inside definition", Some(":"));
                } else {
                    // push the new token onto the definition
                    self.msg
                        .debug("compile_token", "Pushing", Some(&self.token));
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
        let mut loop_stack = Vec::<(&str, usize, usize)>::new();
        let mut conditional_stack = Vec::<(&str, usize, usize)>::new();
        let mut idx = 0; // points to the current token
        while idx < self.new_word_definition.len() {
            let cur_token = self.new_word_definition[idx].clone();
            if let ForthToken::Branch(branch_info) = cur_token {
                let word = &branch_info.word;

                match word.as_str() {
                    "if" => {
                        // put the info on the stack
                        conditional_stack.push(("if", idx, branch_info.branch_id));
                    }
                    "else" => {
                        // pop stack, insert new ZeroEqual Branch token with offset
                        if let Some((_word, place, id)) = conditional_stack.pop() {
                            self.new_word_definition[place] = ForthToken::Branch(BranchInfo::new(
                                "if".to_string(),
                                idx - place,
                                id,
                            ));
                            conditional_stack.push(("else", idx, id));
                        }
                    }
                    "then" => {
                        // pop stack, insert new Unconditional Branch token with offset
                        if let Some((word, place, id)) = conditional_stack.pop() {
                            self.new_word_definition[place] = ForthToken::Branch(BranchInfo::new(
                                word.to_owned(),
                                idx - place,
                                id, // IF statements don't need a branch_id
                            ));
                            self.new_word_definition[idx] =
                                ForthToken::Branch(BranchInfo::new("then".to_string(), 0, id));
                        }
                    }
                    "do" => {
                        // push onto branch_stack
                        loop_stack.push(("do", idx, branch_info.branch_id));
                    }
                    "leave" => {
                        loop_stack.push(("leave", idx, branch_info.branch_id));
                    }
                    "loop" | "+loop" => {
                        let mut branch_id = 0;
                        // Get one record for a start
                        if let Some((name, place, id)) = loop_stack.pop() {
                            let loop_id = id; // capture id for consistency
                            let mut delta: usize = 0; // capture branch-back distance from do
                            if name == "leave" {
                                self.new_word_definition[place] = ForthToken::Branch(
                                    BranchInfo::new("leave".to_owned(), idx - place, loop_id),
                                );
                                branch_id = id; // save for the DO and LOOP records
                            } else {
                                // it must be a DO
                                self.new_word_definition[place] = ForthToken::Branch(
                                    BranchInfo::new("do".to_owned(), idx - place - 1, loop_id),
                                );
                                delta = place;
                            }
                            // if it was LEAVE, we need another record; otherwise proceed to (+)LOOP
                            if branch_id != 0 {
                                // get another record
                                if let Some((_name, place, _id)) = loop_stack.pop() {
                                    self.new_word_definition[place] = ForthToken::Branch(
                                        BranchInfo::new("do".to_owned(), idx - place - 1, loop_id),
                                    );
                                }
                            }
                            // process the LOOP token
                            self.new_word_definition[idx] = ForthToken::Branch(BranchInfo::new(
                                word.clone(),
                                idx - delta + 1,
                                loop_id,
                            ));
                        }
                    }
                    _ => {}
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
                        // first time, (branch_id is not top of control stack) grab limit and first values
                        // and put them on the control stack
                        if self.control_stack.is_empty()
                            || self.control_stack[self.control_stack.len() - 1].id != info.branch_id
                        {
                            // it's our first time
                            // place popped values on the control stack
                            if let (Some(init), Some(end)) = (self.stack.pop(), self.stack.pop()) {
                                self.control_stack.push(ControlFrame::new(
                                    info.branch_id,
                                    init,
                                    end,
                                ));
                            } else {
                                self.msg.error(
                                    "execute_token",
                                    "DO requires END and INIT values on stack",
                                    None::<bool>,
                                );
                                self.abort_flag = true;
                                return (program_counter, jumped);
                            }
                        }
                    }
                    "loop" => {
                        let current_frame = self.control_stack.len() - 1;
                        self.control_stack[current_frame].incr += 1;
                        if self.control_stack[current_frame].incr
                            < self.control_stack[current_frame].end
                        {
                            program_counter -= info.offset;
                            jumped = true;
                        } else {
                            jumped = false;
                            self.control_stack.pop();
                        }
                    }
                    "+loop" => {
                        // get the increment from the calculation stack
                        if let Some(increment) = self.stack.pop() {
                            let current_frame = self.control_stack.len() - 1;
                            self.control_stack[current_frame].incr += increment;
                            if self.control_stack[current_frame].incr
                                < self.control_stack[current_frame].end
                            {
                                program_counter -= info.offset;
                                jumped = true;
                            } else {
                                // program_counter += 1;
                                jumped = false;
                                self.control_stack.pop();
                            }
                        } else {
                            self.msg
                                .error("+loop", "No increment value on stack", None::<bool>);
                            self.abort_flag = true;
                        }
                    }
                    "leave" => {
                        self.control_stack.pop();
                        program_counter += info.offset;
                    }
                    _ => (),
                }
            }
            ForthToken::Definition(_, _) => self.execute_definition(),
            ForthToken::Builtin(op, _) => {
                let index = self.find_builtin(op);
                match index {
                    Some(index) => {
                        let func = &mut self.builtins[index].code;
                        func(self);
                    }
                    None => {}
                }
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
        match &self.token {
            ForthToken::Definition(word_name, _) => {
                match self.find(word_name) {
                    Some(index) => {
                        let definition = self.dictionary[index].clone();
                        match definition {
                            ForthToken::Definition(_n, code) => {
                                while program_counter < code.len() {
                                    if self.abort_flag {
                                        // code.clear();
                                        self.stack.clear();
                                        self.control_stack.clear();
                                        self.abort_flag = false;
                                        break;
                                    } else {
                                        self.token = code[program_counter].clone();
                                        (program_counter, jumped) =
                                            self.execute_token(program_counter, jumped);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    None => {
                        if self.defined_variables.contains_key(word_name) {
                            self.stack.push(self.defined_variables[word_name]); // push the index on the stack
                        } else if self.defined_constants.contains_key(word_name) {
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
                                self.msg.debug("loaded", "processed", Some(&self.token));
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

    fn find_definition(&self, name: &str) -> Option<usize> {
        for i in 0..self.dictionary.len() {
            match &self.dictionary[i] {
                ForthToken::Definition(n, _) => {
                    if n == name {
                        return Some(i);
                    }
                }
                _ => {} // don't care about variables and constants
            }
        }
        None
    }

    fn find_builtin(&self, name: &str) -> Option<usize> {
        for i in 0..self.builtins.len() {
            if self.builtins[i].name == name {
                return Some(i);
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
                        ForthToken::Integer(num) => print!("{num} "),
                        ForthToken::Float(num) => print!("f{num} "),
                        ForthToken::Builtin(op, code) => print!("{op}:{code} "),
                        ForthToken::Definition(name, _def) => print!("{name} "),
                        ForthToken::Branch(info) => {
                            print!("{}:{}:{} ", info.word, info.offset, info.branch_id);
                        }
                        ForthToken::Forward(info) => {
                            print!("{}{} ", info.word, info.tail);
                        }
                        ForthToken::Empty => print!("ForthToken::Empty "),
                        _ => {}
                    }
                }
                println!(";");
            }
            ForthToken::Variable(name, val) => print!("V:{name}:{val} "),
            ForthToken::Constant(name, val) => print!("C:{name}:{val} "),
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

    fn print_control_stack(&self) {
        println!("Control     stack: {:?}", self.control_stack);
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
                    print!("{}:{}:{}: Step> ", info.word, info.offset, info.branch_id);
                }
                ForthToken::Forward(info) => {
                    print!("{}{}: Step> ", info.word, info.tail);
                }
                ForthToken::Builtin(name, code) => print!("{}:{:?}", name, code),
                ForthToken::Definition(name, _def) => print!("{name} "),
                ForthToken::Empty => print!("ForthToken::Empty: Step> "),
                _ => print!("variable or constant???"),
            }
            io::stdout().flush().unwrap();
            match self.parser.reader.read_char() {
                Some('s') => {
                    self.print_stack();
                    self.print_control_stack();
                }
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
