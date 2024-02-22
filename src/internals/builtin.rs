/// Interpreter for builtins
///
/// Set up a table of builtin functions, with names and code

#[allow(dead_code)]
use crate::engine::{BUILTIN, STR_START, TF, TIB_START, VARIABLE};
use crate::{engine::PAD_START, tokenizer::ForthToken};

macro_rules! pop {
    ($self:ident) => {{
        $self.stack_ptr += 1;
        $self.data[$self.stack_ptr - 1]
    }};
}
macro_rules! top {
    ($self:ident) => {{
        $self.data[$self.stack_ptr]
    }};
}
macro_rules! push {
    ($self:ident, $val:expr) => {
        $self.stack_ptr -= 1;
        $self.data[$self.stack_ptr] = $val;
    };
}
macro_rules! stack_ok {
    ($self:ident, $n: expr, $caller: expr) => {
        if $self.stack_ptr <= STACK_START - $n {
            true
        } else {
            $self.msg.error($caller, "Stack underflow", None::<bool>);
            $self.f_abort();
            false
        }
    };
}

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
    pub fn u_insert_variables(&mut self) {
        // install system variables in data area
        // hand craft S-HERE (free string pointer) so write_string() can work
        self.data[0] = 0;
        self.data[1] = STR_START as i64; //
        self.strings[STR_START] = 6 as char; // length of "s-here"
        for (i, c) in "s-here".chars().enumerate() {
            self.strings[i + STR_START + 1] = c;
        }
        self.string_ptr = 3;
        self.data[2] = VARIABLE;
        self.data[3] = (STR_START + 7) as i64; // update the value of S-HERE
        self.data[4] = 0; // back pointer
                          // hand craft HERE, because it's needed by make_word
        let name_pointer = self.u_write_string("here");
        self.data[5] = name_pointer as i64;
        self.data[6] = VARIABLE;
        self.data[7] = 9; // the value of HERE
        self.data[8] = 4; // back pointer
        self.here_ptr = 7; // the address of the HERE variable

        // hand craft CONTEXT, because it's needed by make_word
        self.data[9] = self.u_write_string("context") as i64;
        self.data[10] = VARIABLE;
        self.data[11] = 9;
        self.data[12] = 8; // back pointer
        self.context_ptr = 11;
        self.data[self.here_ptr] = 13;

        self.pad_ptr = self.u_make_variable("pad");
        self.data[self.pad_ptr] = PAD_START as i64;
        self.base_ptr = self.u_make_variable("base");
        self.data[self.base_ptr] = 10; // decimal
                                       //self.tmp_ptr = self.make_variable("tmp");
        self.tib_size_ptr = self.u_make_variable("#tib");
        self.tib_in_ptr = self.u_make_variable(">in");
        self.data[self.tib_in_ptr as usize] = TIB_START as i64 + 1;
        self.hld_ptr = self.u_make_variable("hld");

        self.last_ptr = self.u_make_variable("last");
    }

    /// Insert Forth code into the dictionary
    pub fn u_insert_code(&mut self) {
        // self.u_interpret("2 2 + .");
    }

    fn u_write_string(&mut self, string: &str) -> usize {
        // place a str into string space and update the free pointer string_ptr
        let mut ptr = self.data[self.string_ptr] as usize;
        let result_ptr = ptr;
        self.strings[ptr] = string.len() as u8 as char;
        ptr += 1;
        for (i, c) in string.chars().enumerate() {
            self.strings[ptr + i] = c;
        }
        self.data[self.string_ptr] = (ptr + string.len()) as i64;
        result_ptr
    }

    fn u_make_variable(&mut self, name: &str) -> usize {
        // Create a variable, returning the address and updating the data_ptr
        // build the header for a variable
        let variable_ptr = self.u_make_word(&name, &[VARIABLE, 0]); // install the name
        variable_ptr
    }

    fn u_make_constant(&mut self, name: &str, val: i64) -> usize {
        // Create a constant
        // build the header for a constant
        let const_ptr = self.u_make_word(name, &[val]); // install the name
        const_ptr
    }

    fn u_make_word(&mut self, name: &str, args: &[i64]) -> usize {
        // install a new word with provided name and arguments
        // back link is already in place
        // place it HERE
        // update HERE and LAST
        // return HERE
        let back = self.data[self.here_ptr] as usize - 1; // the top-of-stack back pointer's location
        let mut ptr = back + 1;
        self.data[ptr] = self.u_write_string(name) as i64;
        for val in args {
            ptr += 1;
            self.data[ptr] = *val;
        }
        ptr += 1;
        self.data[ptr] = back as i64; // the new back pointer
        self.data[self.here_ptr] = ptr as i64 + 1; // top of the stack = HERE
        self.data[self.context_ptr] = back as i64 + 1; // context is the name_pointer field of this word
        back + 2 // address of first parameter field
    }

    fn u_add(&mut self, name: &str, code: for<'a> fn(&'a mut TF), doc: &str) {
        self.builtins
            .push(BuiltInFn::new(name.to_owned(), code, doc.to_string()));
        // now build the DATA space record
        self.u_make_word(name, &[BUILTIN, self.builtins.len() as i64 - 1]);
    }

    pub fn add_builtins(&mut self) {
        self.u_add("+", TF::f_plus, "+ ( j k -- j+k ) Push j+k on the stack");
        self.u_add("-", TF::f_minus, "- ( j k -- j+k ) Push j-k on the stack");
        self.u_add("*", TF::f_times, "* ( j k -- j-k ) Push  -k on the stack");
        self.u_add("/", TF::f_divide, "/ ( j k -- j/k ) Push j/k on the stack");
        self.u_add("mod", TF::f_mod, "mod ( j k -- j/k ) Push j%k on the stack");
        self.u_add(
            "<",
            TF::f_less,
            "( j k -- j/k ) If j < k push true else false",
        );
        self.u_add(
            ".",
            TF::f_dot,
            ". ( n -- ) Pop the top of the stack and print it, followed by a space",
        );
        self.u_add(
            "true",
            TF::f_true,
            "true ( -- -1 ) Push the canonical true value on the stack.",
        );
        self.u_add(
            "false",
            TF::f_false,
            "false ( -- 0 ) Push the canonical false value on the stack",
        );
        self.u_add(
            "=",
            TF::f_equal,
            "= ( j k -- b ) If j == k push true else false",
        );
        self.u_add(
            "0=",
            TF::f_0equal,
            "0= ( j -- b ) If j == 0 push true else false",
        );
        self.u_add(
            "0<",
            TF::f_0less,
            "( j k -- j/k ) If j < 0 push true else false",
        );
        self.u_add(
            ".s",
            TF::f_dot_s,
            ".s ( -- ) Print the contents of the calculation stack",
        );
        self.u_add("cr", TF::f_cr, "cr ( -- ) Print a newline");
        self.u_add(
            "show-stack",
            TF::f_show_stack,
            "show-stack ( -- ) Display the stack at the end of each line of console input",
        );
        self.u_add(
            "hide-stack",
            TF::f_hide_stack,
            "hide-stack ( -- ) Turn off automatic stack display",
        );
        self.u_add(
            ".s\"",
            TF::f_dot_s_quote,
            ".s\" Print the contents of the pad",
        );
        self.u_add(
            "emit",
            TF::f_emit,
            "emit: ( c -- ) if printable, sends character c to the terminal",
        );
        self.u_add(
            "flush",
            TF::f_flush,
            "flush: forces pending output to appear on the terminal",
        );
        self.u_add("clear", TF::f_clear, "clear: resets the stack to empty");
        self.u_add(":", TF::f_colon, ": starts a new definition");
        self.u_add("bye", TF::f_bye, "bye: exits to the operating system");
        self.u_add(
            "words",
            TF::f_words,
            "words: Lists all defined words to the terminal",
        );
        self.u_add(
            "dup",
            TF::f_dup,
            "dup ( n -- n n ) Push a second copy of the top of stack",
        );
        self.u_add(
            "drop",
            TF::f_drop,
            "drop ( n --  ) Pop the top element off the stack",
        );
        self.u_add(
            "swap",
            TF::f_swap,
            "swap ( m n -- n m ) Reverse the order of the top two stack elements",
        );
        self.u_add(
            "over",
            TF::f_over,
            "over ( m n -- m n m ) Push a copy of the second item on the stack on to",
        );
        self.u_add(
            "rot",
            TF::f_rot,
            "rot ( i j k -- j k i ) Move the third stack item to the top",
        );
        self.u_add(
            "and",
            TF::f_and,
            "and ( a b -- a & b ) Pop a and b, returning the logical and",
        );
        self.u_add(
            "or",
            TF::f_or,
            "or ( a b -- a | b ) Pop a and b, returning the logical or",
        );
        self.u_add("@", TF::f_get, "@: ( a -- v ) Pushes variable a's value");
        self.u_add("!", TF::f_store, "!: ( v a -- ) stores v at address a");
        self.u_add("i", TF::f_i, "Pushes the current FOR - NEXT loop index");
        self.u_add("j", TF::f_j, "Pushes the second-level (outer) loop index");
        self.u_add(
            "abort",
            TF::f_abort,
            "abort ( -- ) Ends execution of the current word and clears the stack",
        );
        self.u_add(
            "see-all",
            TF::f_see_all,
            "see-all: Prints the definitions of known words",
        );
        self.u_add(
            "depth",
            TF::f_stack_depth,
            "depth: Pushes the current stack depth",
        );
        self.u_add(
            "key",
            TF::f_key,
            "key ( -- c ) Get a character from the terminal",
        );
        self.u_add("r/w", TF::f_r_w, "");
        self.u_add("r/o", TF::f_r_o, "");
        self.u_add(
            "include-file",
            TF::f_include_file,
            "include-file ( a -- ) Taking the TOS as a pointer to 
        a filename (string), load a file of source code",
        );
        self.u_add("dbg", TF::f_dbg, "");
        self.u_add(
            "debuglevel",
            TF::f_debuglevel,
            "debuglevel ( -- ) Displays the current debug level",
        );
        self.u_add("step-on", TF::f_step_on, "");
        self.u_add("step-off", TF::f_step_off, "");
        self.u_add(
            ">r",
            TF::f_to_r,
            ">r ( n -- ) Pop stack and push value to return stack",
        );
        self.u_add(
            "r>",
            TF::f_r_from,
            "r> ( -- n ) Pop return stack and push value to calculation stack",
        );
        self.u_add(
            "r@",
            TF::f_r_get,
            "r@ ( -- n ) Push the value on the top of the return stack to the calculation stack",
        );
        self.u_add(
            "immediate",
            TF::f_immediate,
            "immediate sets the immediate flag on the most recently defined word",
        );
        self.u_add("[", TF::f_lbracket, "[ ( -- ) Exit compile mode");
        self.u_add("]", TF::f_rbracket, "] ( -- ) Enter compile mode");
        self.u_add(
            "quit",
            TF::f_quit,
            "quit ( -- ) Outer interpreter that repeatedly reads input lines and runs them",
        );
        self.u_add(
            "execute",
            TF::f_execute,
            "execute: interpret the word whose address is on the stack",
        );
        self.u_add(
            "interpret",
            TF::f_eval,
            "interpret: Interprets one line of Forth",
        );
        self.u_add(
            "number?",
            TF::f_number_q,
            "number? ( a -- n T | a F ) tests a string to see if it's a number;
            leaves n and flag on the stack: true if number is ok.",
        );
        self.u_add(
            "?unique",
            TF::f_q_unique,
            "?unique ( a -- b ) tests to see if the name TOS points to is in the dictionary",
        );
        self.u_add(
            "find",
            TF::f_find,
            "FIND (s -- a | F ) Search the dictionary for the token indexed through s. 
        Return it's address or FALSE if not found",
        );
        self.u_add(
            "'",
            TF::f_tick,
            "' (tick): searches the dictionary for a (postfix) word",
        );
        self.u_add(
            "query",
            TF::f_query,
            "query ( -- ) Read a line from the console into TIB",
        );
        self.u_add(
            "accept",
            TF::f_accept,
            "accept ( b l1 -- b l2 ) Read up to l1 characters into the buffer at b.
        Return the pointer to the buffer and the actual number of characters read.",
        );
        self.u_add(
            "text",
            TF::f_text,
            "TEXT ( -- ) Get a space-delimited token from the TIB, place in PAD",
        );
        self.u_add(
            "parse",
            TF::f_text,
            "PARSE ( c -- b u ) Get a c-delimited token from TIB, 
        and return counted string in PAD",
        );
        self.u_add(
            "(parse)",
            TF::f_text,
            "(parse) - b u c -- b u delta ) return the location of a delimited token in string space",
        );
        self.u_add(
            "s\"",
            TF::f_s_quote,
            "s\" Place the following string in the PAD",
        );
        self.u_add(
            "type",
            TF::f_type,
            "type: print a string using pointer on stack",
        );
        self.u_add("recurse", TF::f_recurse, "recurse")
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
        self.eval_ptr = self.add_variable("'eval", 0);
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

    pub fn get_string(&mut self, addr: usize) -> String {
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

    pub fn set_string(&mut self, addr: usize, new_val: &str) {
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
