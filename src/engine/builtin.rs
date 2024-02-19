/// Interpreter for builtins
///
/// Set up a table of builtin functions, with names and code

#[allow(dead_code)]
use crate::engine::*;
use crate::messages::DebugLevel;
use crate::tokenizer::{is_integer, ForthToken};
use std::cmp::min;
use std::io::{self, Write};

pub trait BuiltinCall {
    fn call(&mut self);
}

pub struct BuiltInFn {
    pub name: String,
    pub code: for<'a> fn(&'a mut TF),
    pub doc: String,
}

impl BuiltinCall for BuiltInFn {
    fn call(&mut self) {}
}

impl BuiltInFn {
    pub fn new(name: String, code: for<'a> fn(&'a mut TF), doc: String) -> BuiltInFn {
        BuiltInFn { name, code, doc }
    }
}

macro_rules! pop2_push1 {
    // Helper macro
    ($self:ident, $word:expr, $expression:expr) => {
        if let Some((j, k)) = $self.pop_two(&$word) {
            $self.stack.push($expression(k, j));
        }
    };
}
macro_rules! pop1_push1 {
    // Helper macro
    ($self:ident, $word:expr, $expression:expr) => {
        if let Some(x) = $self.pop_one(&$word) {
            $self.stack.push($expression(x));
        }
    };
}
macro_rules! pop1 {
    ($self:ident, $word:expr, $code:expr) => {
        if let Some(x) = $self.pop_one(&$word) {
            $code(x);
        }
    };
}

impl TF {
    pub fn f_insert_variables(&mut self) {
        // install system variables in data area
        // hand craft HERE, because it's needed by make_word
        self.data[0] = 0; // null pointer
        self.data[1] = 4 as i64; //
        for (i, c) in "here".char_indices() {
            self.data[i + 2] = c as i64;
        }
        self.data[6] = 7; // the value of HERE
        self.data[7] = 0; // back pointer
        self.here_ptr = 6; // the address of the HERE variable

        // hand craft CONTEXT, because it's needed by make_word
        self.data[8] = 7 as i64;
        for (i, c) in "context".char_indices() {
            self.data[i + 9] = c as i64;
        }
        self.data[16] = 8;
        self.data[17] = 7; // back pointer
        self.context_ptr = 8;
        self.data[self.here_ptr] = 17;

        /*         self.base_ptr = self.make_variable("base");
               self.data[self.base_ptr] = 10;
               self.tmp_ptr = self.make_variable("tmp");
               self.tib_in_ptr = self.make_variable(">in");
               self.data[self.tib_in_ptr as usize] = TIB_START as i32;
               self.hld_ptr = self.make_variable("hld");

               self.last_ptr = self.make_variable("last");
               self.data[self.here_ptr] as usize
        */
    }

    fn f_insert_variable(&mut self) {}

    fn add(&mut self, name: &str, code: for<'a> fn(&'a mut TF), doc: &str) {
        self.builtins
            .push(BuiltInFn::new(name.to_owned(), code, doc.to_string()));
        // now build the DATA space record
    }

    pub fn add_builtins(&mut self) {
        // add the builtins to the builtin dictionary
        self.add("+", TF::f_plus, "+ ( j k -- j+k ) Push j+k on the stack");
        self.add("-", TF::f_minus, "- ( j k -- j+k ) Push j-k on the stack");
        self.add("*", TF::f_times, "* ( j k -- j-k ) Push  -k on the stack");
        self.add("/", TF::f_divide, "/ ( j k -- j/k ) Push j/k on the stack");
        self.add("mod", TF::f_mod, "mod ( j k -- j/k ) Push j%k on the stack");
        self.add(
            "<",
            TF::f_less,
            "( j k -- j/k ) If j < k push true else false",
        );
        self.add(
            ".",
            TF::f_dot,
            ". ( n -- ) Pop the top of the stack and print it, followed by a space",
        );
        self.add(
            "true",
            TF::f_true,
            "true ( -- -1 ) Push the canonical true value on the stack.",
        );
        self.add(
            "false",
            TF::f_false,
            "false ( -- 0 ) Push the canonical false value on the stack",
        );
        self.add(
            "=",
            TF::f_equal,
            "= ( j k -- b ) If j == k push true else false",
        );
        self.add(
            "0=",
            TF::f_0equal,
            "0= ( j -- b ) If j == 0 push true else false",
        );
        self.add(
            "0<",
            TF::f_0less,
            "( j k -- j/k ) If j < 0 push true else false",
        );
        self.add(
            ".s",
            TF::f_dot_s,
            ".s ( -- ) Print the contents of the calculation stack",
        );
        self.add("cr", TF::f_cr, "cr ( -- ) Print a newline");
        self.add(
            "show-stack",
            TF::f_show_stack,
            "show-stack ( -- ) Display the stack at the end of each line of console input",
        );
        self.add(
            "hide-stack",
            TF::f_hide_stack,
            "hide-stack ( -- ) Turn off automatic stack display",
        );
        self.add(
            ".s\"",
            TF::f_dot_s_quote,
            ".s\" Print the contents of the pad",
        );
        self.add(
            "emit",
            TF::f_emit,
            "emit: ( c -- ) if printable, sends character c to the terminal",
        );
        self.add(
            "flush",
            TF::f_flush,
            "flush: forces pending output to appear on the terminal",
        );
        self.add("clear", TF::f_clear, "clear: resets the stack to empty");
        self.add(":", TF::f_colon, ": starts a new definition");
        self.add("bye", TF::f_bye, "bye: exits to the operating system");
        self.add(
            "words",
            TF::f_words,
            "words: Lists all defined words to the terminal",
        );
        self.add(
            "dup",
            TF::f_dup,
            "dup ( n -- n n ) Push a second copy of the top of stack",
        );
        self.add(
            "drop",
            TF::f_drop,
            "drop ( n --  ) Pop the top element off the stack",
        );
        self.add(
            "swap",
            TF::f_swap,
            "swap ( m n -- n m ) Reverse the order of the top two stack elements",
        );
        self.add(
            "over",
            TF::f_over,
            "over ( m n -- m n m ) Push a copy of the second item on the stack on to",
        );
        self.add(
            "rot",
            TF::f_rot,
            "rot ( i j k -- j k i ) Move the third stack item to the top",
        );
        self.add(
            "and",
            TF::f_and,
            "and ( a b -- a & b ) Pop a and b, returning the logical and",
        );
        self.add(
            "or",
            TF::f_or,
            "or ( a b -- a | b ) Pop a and b, returning the logical or",
        );
        self.add("@", TF::f_get, "@: ( a -- v ) Pushes variable a's value");
        self.add("!", TF::f_store, "!: ( v a -- ) stores v at address a");
        self.add("i", TF::f_i, "Pushes the current FOR - NEXT loop index");
        self.add("j", TF::f_j, "Pushes the second-level (outer) loop index");
        self.add(
            "abort",
            TF::f_abort,
            "abort ( -- ) Ends execution of the current word and clears the stack",
        );
        self.add(
            "see-all",
            TF::f_see_all,
            "see-all: Prints the definitions of known words",
        );
        self.add(
            "depth",
            TF::f_stack_depth,
            "depth: Pushes the current stack depth",
        );
        self.add(
            "key",
            TF::f_key,
            "key ( -- c ) Get a character from the terminal",
        );
        self.add("r/w", TF::f_r_w, "");
        self.add("r/o", TF::f_r_o, "");
        self.add(
            "include-file",
            TF::f_include_file,
            "include-file ( a -- ) Taking the TOS as a pointer to 
        a filename (string), load a file of source code",
        );
        self.add("dbg", TF::f_dbg, "");
        self.add(
            "debuglevel",
            TF::f_debuglevel,
            "debuglevel ( -- ) Displays the current debug level",
        );
        self.add("step-on", TF::f_step_on, "");
        self.add("step-off", TF::f_step_off, "");
        self.add(
            ">r",
            TF::f_to_r,
            ">r ( n -- ) Pop stack and push value to return stack",
        );
        self.add(
            "r>",
            TF::f_r_from,
            "r> ( -- n ) Pop return stack and push value to calculation stack",
        );
        self.add(
            "r@",
            TF::f_r_get,
            "r@ ( -- n ) Push the value on the top of the return stack to the calculation stack",
        );
        self.add("[", TF::f_lbracket, "[ ( -- ) Exit compile mode");
        self.add("]", TF::f_rbracket, "] ( -- ) Enter compile mode");
        self.add(
            "quit",
            TF::f_quit,
            "quit ( -- ) Outer interpreter that repeatedly reads input lines and runs them",
        );
        self.add(
            "execute",
            TF::f_execute,
            "execute: interpret the word whose address is on the stack",
        );
        self.add(
            "interpret",
            TF::f_interpret,
            "interpret: Interprets one line of Forth",
        );
        self.add(
            "number?",
            TF::f_number_q,
            "number?: tests a string to see if it's a number;
            leaves n and flag on the stack: true if number is ok.",
        );
        self.add(
            "?unique",
            TF::f_q_unique,
            "?unique ( a -- b ) tests to see if the name TOS points to is in the dictionary",
        );
        self.add(
            "'",
            TF::f_tick,
            "' (tick): searches the dictionary for a (postfix) word",
        );
        self.add("accept", TF::f_accept, "");
        self.add("text", TF::f_text, "");
        self.add(
            "type",
            TF::f_type,
            "type: print a string using pointer on stack",
        );
    }

    fn f_plus(&mut self) {
        pop2_push1!(self, "+", |a, b| a + b);
    }
    fn f_minus(&mut self) {
        pop2_push1!(self, "-", |a, b| a - b);
    }
    fn f_times(&mut self) {
        pop2_push1!(self, "*", |a, b| a * b);
    }
    fn f_divide(&mut self) {
        pop2_push1!(self, "/", |a, b| a / b);
    }
    fn f_mod(&mut self) {
        pop2_push1!(self, "mod", |a, b| a % b);
    }
    fn f_less(&mut self) {
        pop2_push1!(self, "<", |a, b| if a < b { -1 } else { 0 });
    }
    fn f_dot(&mut self) {
        pop1!(self, ".", |a| print!("{a} "));
    }
    fn f_true(&mut self) {
        self.stack.push(-1);
    }
    fn f_false(&mut self) {
        self.stack.push(0);
    }
    fn f_equal(&mut self) {
        pop2_push1!(self, "=", |a, b| if a == b { -1 } else { 0 });
    }
    fn f_0equal(&mut self) {
        pop1_push1!(self, "0=", |a| if a == 0 { -1 } else { 0 });
    }
    fn f_0less(&mut self) {
        pop1_push1!(self, "0<", |a| if a < 0 { -1 } else { 0 });
    }
    fn f_dot_s(&mut self) {
        println!("{:?}", self.stack);
    }
    fn f_cr(&mut self) {
        println!("");
    }
    fn f_show_stack(&mut self) {
        self.show_stack = true;
    }
    fn f_hide_stack(&mut self) {
        self.show_stack = false;
    }
    fn f_dot_s_quote(&mut self) {
        print!("{:?}", self.get_string_var(self.pad_ptr));
    }
    fn f_emit(&mut self) {
        match self.stack.pop() {
            Some(n) => {
                if (0x20..=0x7f).contains(&n) {
                    print!("{}", n as u8 as char);
                } else {
                    self.msg.error("EMIT", "Arg out of range", Some(n));
                }
            }
            None => {}
        }
    }
    fn f_flush(&mut self) {
        io::stdout().flush().unwrap();
    }
    fn f_clear(&mut self) {
        self.stack.clear()
    }
    fn f_colon(&mut self) {
        self.set_compile_mode(true);
    }
    fn f_bye(&mut self) {
        self.set_exit_flag();
    }
    fn f_words(&mut self) {
        for word in self.dictionary.iter() {
            match word {
                ForthToken::Definition(name, _) => print!("{name} "),
                _ => {} // ignore other token types
            }
        }
        println!();
    }
    fn f_dup(&mut self) {
        if let Some(top) = self.stack.last() {
            self.stack.push(*top);
        } else {
            self.msg
                .warning("DUP", "Error - DUP: Stack is empty.", None::<bool>);
        }
    }
    fn f_drop(&mut self) {
        pop1!(self, "drop", |_a| ());
    }
    fn f_swap(&mut self) {
        if self.stack.len() > 1 {
            let a = self.stack[self.stack.len() - 1];
            let b = self.stack[self.stack.len() - 2];
            self.stack.pop();
            self.stack.pop();
            self.stack.push(a);
            self.stack.push(b);
        } else {
            self.msg
                .warning("SWAP", "Too few elements on stack.", None::<bool>);
        }
    }
    fn f_over(&mut self) {
        if self.stack_underflow("OVER", 2) {
            self.set_abort_flag(true);
        } else {
            let item = self.stack.get(self.stack.len() - 2);
            match item {
                Some(item) => {
                    self.stack.push(*item);
                }
                None => {
                    self.set_abort_flag(true);
                }
            }
        }
    }
    fn f_rot(&mut self) {
        if self.stack_underflow("ROT", 3) {
            self.set_abort_flag(true);
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
    fn f_and(&mut self) {
        if !self.stack_underflow("AND", 2) {
            if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                self.stack.push(a & b);
            }
        }
    }
    fn f_or(&mut self) {
        if !self.stack_underflow("OR", 2) {
            if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                self.stack.push(a | b);
            }
        }
    }
    fn f_get(&mut self) {
        if !self.stack_underflow("@", 1) {
            if let Some(adr) = self.stack.pop() {
                let value = self.get_var(adr as usize);
                self.stack.push(value as i64);
            }
        }
    }
    fn f_store(&mut self) {
        if !self.stack_underflow("!", 2) {
            if let (Some(addr), Some(val)) = (self.stack.pop(), self.stack.pop()) {
                self.set_var(addr as usize, val);
            }
        }
    }
    fn f_to_r(&mut self) {
        pop1!(self, ">r", |n| self.return_stack.push(n));
    }
    fn f_r_from(&mut self) {
        if let Some(n) = self.return_stack.pop() {
            self.stack.push(n);
        } else {
            self.msg.error("r>", "Return stack underflow", None::<bool>);
        }
    }
    fn f_r_get(&mut self) {
        if self.return_stack.len() > 0 {
            self.stack.push(*self.return_stack.last().unwrap());
        } else {
            self.msg.error("r@", "Return stack underflow", None::<bool>);
        }
    }
    fn f_i(&mut self) {
        // print the index of the current top-level loop
        if self.return_stack.is_empty() {
            self.msg.warning(
                "I",
                "Can only be used inside a DO .. LOOP structure",
                None::<bool>,
            );
        } else {
            self.stack
                .push(self.return_stack[self.return_stack.len() - 1]);
        }
    }
    fn f_j(&mut self) {
        // print the index of the current second-level (outer) loop
        if self.return_stack.len() < 2 {
            self.msg.warning(
                "I",
                "Can only be used inside a nested DO .. LOOP structure",
                None::<bool>,
            );
        } else {
            self.stack
                .push(self.return_stack[self.return_stack.len() - 2]);
        }
    }
    fn f_lbracket(&mut self) {
        self.set_compile_mode(false);
    }
    fn f_rbracket(&mut self) {
        self.set_compile_mode(true);
    }
    pub fn f_abort(&mut self) {
        // empty the stack, reset any pending operations, and return to the prompt
        self.msg
            .warning("ABORT", "Terminating execution", None::<bool>);
        self.stack.clear();
        self.set_abort_flag(true);
    }
    pub fn f_quit(&mut self) {
        self.return_stack.clear();
        self.set_program_counter(0);
        self.f_abort();
        loop {
            if self.should_exit() {
                break;
            } else {
                self.stack.push(132);
                self.f_accept(); // get a line from the terminal
                self.f_interpret(); // interpret the contents of the line
                println!("ok");
            }
        }
    }
    fn f_execute(&mut self) {
        // execute a word with addr on the stack
        match self.stack.pop() {
            Some(addr) => {
                if addr < 999 {
                    self.execute_word(addr as usize);
                } else {
                    self.execute_builtin(addr as usize - 1000);
                }
            }
            None => {}
        }
    }
    fn f_interpret(&mut self) {
        // process a line of tokens
        loop {
            if self.get_var(self.tib_in_ptr) >= self.get_var(self.tib_size_ptr) {
                // no more tokens on this line
                return;
            } else {
                self.f_tick(); // grabs the next word and searches the dict
                match self.stack.pop() {
                    Some(val) => {
                        if val > 0 {
                            self.stack.push(val);
                            self.f_execute();
                        } else {
                            // it's not a word, but could be a number
                            self.stack.push(self.pad_ptr as i64);
                            self.f_number_q(); // tries to convert the pad string
                            match self.stack.pop() {
                                Some(b) => {
                                    if b == 0 {
                                        // forth false flag
                                        let _ = self.stack.pop(); // wasn't a number
                                    }
                                } // number is on the stack
                                None => {} // error condition
                            }
                        }
                    }
                    None => {
                        self.f_abort();
                        return;
                    }
                }
            }
        }
    }
    fn f_number_q(&mut self) {
        // try to convert the number with string address on the stack
        let mut result = 0;
        let mut flag = 0;
        match self.stack.pop() {
            Some(addr) => {
                let numtext = self.get_string_var(addr as usize);
                if is_integer(numtext.as_str()) {
                    result = numtext.parse().unwrap();
                    flag = -1; // valid number
                }
            }
            None => {}
        }
        self.stack.push(result);
        self.stack.push(flag);
    }
    fn f_q_unique(&mut self) {
        // see if a word is unique. Result boolean on stack
        match self.stack.pop() {
            Some(v) => self.stack.push(TRUE),
            None => self.stack.push(FALSE),
        }
    }
    fn f_tick(&mut self) {
        // looks for a (postfix) word in the dictionary
        // places it's execution token / address on the stack
        // builtin addresses have been bumped up by 1000 to distinguish them
        self.f_s_quote(); // gets a string and places it in PAD
                          // search the dictionary
        let token = self.get_string_var(self.pad_ptr);
        match self.find(&token) {
            Some(idx) => self.stack.push(idx as i64),
            None => self.stack.push(0),
        }
    }
    fn f_accept(&mut self) {
        // get a new line of input and initialize the pointer variable
        match self.stack.pop() {
            Some(max_len) => match self.parser.reader.get_line(&"".to_owned(), false) {
                Some(mut line) => {
                    let length = min(line.len() - 1, max_len as usize) as usize;
                    line = line[..length].to_owned();
                    self.set_string_var(self.tib_ptr, &line);
                    self.set_var(self.tib_in_ptr, 0);
                    self.set_var(self.tib_size_ptr, length as i64);
                }
                None => {
                    self.msg
                        .error("ACCEPT", "Unable to read from input", None::<bool>);
                    self.f_abort();
                }
            },
            None => self
                .msg
                .error("ACCEPT", "Required length not on stack", None::<bool>),
        }
    }
    fn f_text(&mut self) {
        // take delimiter from stack; grab string from TIB
        // need to check if TIB is empty
        // if delimiter = 1, get the rest of the TIB
        match self.stack.pop() {
            Some(d) => {
                let delim = d as u8;
                let in_p = self.get_var(self.tib_in_ptr);
                let mut i = in_p as usize;
                let mut j = i;
                let line = &self.get_string_var(self.tib_ptr);
                if delim as u8 == 1 {
                    // get the rest of the line by setting j to the end
                    j = self.get_var(self.tib_size_ptr) as usize;
                } else {
                    while i < line.len() && line.as_bytes()[i] == delim {
                        // skip leading delims
                        i += 1;
                    }
                    j = i;
                    while j < line.len() && line.as_bytes()[j] != delim {
                        j += 1;
                    }
                }
                self.set_var(self.tib_in_ptr, j as i64);
                let token = line[i..j].to_owned(); // does not include j!
                self.set_string_var(self.pad_ptr, token.as_str());
            }
            None => self
                .msg
                .error("TEXT", "No delimiter on stack", None::<bool>), // stack was empty! error
        }
    }
    fn f_type(&mut self) {
        // print a string, found via pointer on stack
        match self.stack.pop() {
            Some(addr) => {
                let text = self.get_string_var(addr as usize);
                print!("{text}");
            }
            None => {}
        }
    }
    fn f_s_quote(&mut self) {
        // get a string and place it in PAD
        self.stack.push(' ' as i64);
        self.f_text(); // gets the string
    }
    fn f_see_all(&mut self) {
        for i in 0..self.dictionary.len() {
            self.see_word(i);
        }
    }
    fn f_stack_depth(&mut self) {
        self.stack.push(self.stack.len() as i64);
    }
    fn f_key(&mut self) {
        let c = self.parser.reader.read_char();
        match c {
            Some(c) => self.stack.push(c as i64),
            None => self
                .msg
                .error("KEY", "unable to get char from input stream", None::<bool>),
        }
    }
    fn f_r_w(&mut self) {
        self.file_mode = FileMode::ReadWrite;
    }
    fn f_r_o(&mut self) {
        self.file_mode = FileMode::ReadOnly;
    }
    fn f_include_file(&mut self) {
        self.loaded();
    }
    fn f_dbg(&mut self) {
        match self.stack.pop() {
            Some(0) => self.msg.set_level(DebugLevel::Error),
            Some(1) => self.msg.set_level(DebugLevel::Warning),
            Some(2) => self.msg.set_level(DebugLevel::Info),
            _ => self.msg.set_level(DebugLevel::Debug),
        }
    }
    fn f_debuglevel(&mut self) {
        println!("DebugLevel is {:?}", self.msg.get_level());
    }
    fn f_step_on(&mut self) {
        self.step_mode = true;
    }
    fn f_step_off(&mut self) {
        self.step_mode = false;
    }

    /// ADD SYSTEM VARIABLES

    pub fn add_variables(&mut self) {
        self.pc_ptr = self.add_variable("pc", 0); // program counter
        self.compile_ptr = self.add_variable("compile?", 0); // compile mode
        self.abort_ptr = self.add_variable("abort?", 0); // abort flag
        self.tib_ptr = self.add_string_var("tib", "");
        self.tib_size_ptr = self.add_variable("#tib", 0); // length of text input buffer
        self.tib_in_ptr = self.add_variable(">in", 0); // current position in input buffer
        self.pad_ptr = self.add_string_var("pad", "");
    }

    fn add_variable(&mut self, name: &str, val: i64) -> usize {
        self.dictionary
            .push(ForthToken::Variable(name.to_owned(), val));
        self.dictionary.len() - 1
    }

    fn add_string_var(&mut self, name: &str, val: &str) -> usize {
        self.dictionary
            .push(ForthToken::StringVar(name.to_owned(), val.to_owned()));
        self.dictionary.len() - 1
    }

    pub fn set_var(&mut self, addr: usize, new_val: i64) {
        // set the variable at addr to val
        let address = addr.max(0) as usize;
        if address < self.dictionary.len() {
            let var = &self.dictionary[addr];
            match var {
                ForthToken::Variable(name, _v) => {
                    self.dictionary[addr] = ForthToken::Variable(name.to_owned(), new_val)
                }
                _ => self
                    .msg
                    .error("sysvar_set", "Does not point to variable", Some(addr)),
            }
        }
    }

    pub fn get_var(&mut self, addr: usize) -> i64 {
        // gets the current value of the variable at addr
        let address = addr.max(0) as usize;
        if address < self.dictionary.len() {
            let var = &self.dictionary[addr];
            match var {
                ForthToken::Variable(_, value) => *value,
                _ => {
                    self.msg
                        .error("sysvar-get", "Does not point to variable", Some(addr));
                    self.set_abort_flag(true);
                    0
                }
            }
        } else {
            self.set_abort_flag(true);
            0
        }
    }

    pub fn get_string_var(&mut self, addr: usize) -> String {
        // gets the current value of the variable at addr
        let address = addr.max(0) as usize;
        if address < self.dictionary.len() {
            let var = &self.dictionary[addr];
            match var {
                ForthToken::StringVar(_, value) => value.clone(),
                _ => {
                    self.msg
                        .error("stringvar-get", "Does not point to variable", Some(addr));
                    self.set_abort_flag(true);
                    "".to_string()
                }
            }
        } else {
            self.set_abort_flag(true);
            "".to_string()
        }
    }

    pub fn set_string_var(&mut self, addr: usize, new_val: &str) {
        // set the variable at addr to val
        let address = addr.max(0) as usize;
        if address < self.dictionary.len() {
            let var = &self.dictionary[addr];
            match var {
                ForthToken::StringVar(name, _v) => {
                    let name = name.clone();
                    self.dictionary[addr] = ForthToken::StringVar(name, new_val.to_string())
                }
                _ => self
                    .msg
                    .error("stringvar_set", "Does not point to variable", Some(addr)),
            }
        }
    }
}