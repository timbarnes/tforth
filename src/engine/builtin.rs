/// Interpreter for builtins
///
/// Set up a table of builtin functions, with names and code

#[allow(dead_code)]
use crate::engine::{FileMode, TF};
use crate::messages::DebugLevel;
use crate::tokenizer::ForthToken;
use std::io::{self, Write};

pub trait BuiltinCall {
    fn call(&mut self);
}

pub struct BuiltInFn {
    pub name: String,
    pub code: for<'a> fn(&'a mut TF),
}

impl BuiltinCall for BuiltInFn {
    fn call(&mut self) {}
}

impl BuiltInFn {
    pub fn new(name: String, code: for<'a> fn(&'a mut TF)) -> BuiltInFn {
        BuiltInFn { name, code }
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
    fn add(&mut self, name: &str, code: for<'a> fn(&'a mut TF)) {
        self.builtins.push(BuiltInFn::new(name.to_owned(), code));
    }

    pub fn add_builtins(&mut self) {
        // add the builtins to the builtin dictionary
        self.add("+", TF::f_plus);
        self.add("-", TF::f_minus);
        self.add("*", TF::f_times);
        self.add("/", TF::f_divide);
        self.add("mod", TF::f_mod);
        self.add("<", TF::f_less);
        self.add(".", TF::f_dot);
        self.add("true", TF::f_true);
        self.add("false", TF::f_false);
        self.add("=", TF::f_equal);
        self.add("0=", TF::f_0equal);
        self.add("0<", TF::f_0less);
        self.add(".s", TF::f_dot_s);
        self.add("cr", TF::f_cr);
        self.add("show-stack", TF::f_show_stack);
        self.add("hide-stack", TF::f_hide_stack);
        self.add(".s\"", TF::f_dot_s_quote);
        self.add("emit", TF::f_emit);
        self.add("flush", TF::f_flush);
        self.add("clear", TF::f_clear);
        self.add(":", TF::f_colon);
        self.add("bye", TF::f_bye);
        self.add("words", TF::f_words);
        self.add("dup", TF::f_dup);
        self.add("drop", TF::f_drop);
        self.add("swap", TF::f_swap);
        self.add("over", TF::f_over);
        self.add("rot", TF::f_rot);
        self.add("and", TF::f_and);
        self.add("or", TF::f_or);
        self.add("@", TF::f_get);
        self.add("!", TF::f_store);
        self.add("i", TF::f_i);
        self.add("j", TF::f_j);
        self.add("abort", TF::f_abort);
        self.add("see-all", TF::f_see_all);
        self.add("depth", TF::f_stack_depth);
        self.add("key", TF::f_key);
        self.add("r/w", TF::f_r_w);
        self.add("r/o", TF::f_r_o);
        self.add("loaded", TF::f_loaded);
        self.add("dbg", TF::f_dbg);
        self.add("debuglevel", TF::f_debuglevel);
        self.add("step-on", TF::f_step_on);
        self.add("step-off", TF::f_step_off);
        self.add(">r", TF::f_to_r);
        self.add("r>", TF::f_r_from);
        self.add("r@", TF::f_r_get);
        self.add("[", TF::f_lbracket);
        self.add("]", TF::f_rbracket);
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
        print!("{:?}", self.text);
    }
    fn f_emit(&mut self) {
        if !self.stack_underflow("echo", 1) {
            let n = self.stack.pop();
            if let Some(n) = n {
                if (0x20..=0x7f).contains(&n) {
                    let c = n as u8 as char;
                    print!("{}", c);
                } else {
                    self.msg.error("EMIT", "Arg out of range", Some(n));
                }
            }
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
                let value = self.var_get(adr as usize);
                self.stack.push(value as i64);
            }
        }
    }
    fn f_store(&mut self) {
        if !self.stack_underflow("!", 2) {
            if let (Some(addr), Some(val)) = (self.stack.pop(), self.stack.pop()) {
                self.var_set(addr as usize, val);
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
    fn f_abort(&mut self) {
        // empty the stack, reset any pending operations, and return to the prompt
        self.msg
            .warning("ABORT", "Terminating execution", None::<bool>);
        self.stack.clear();
        self.return_stack.clear();
        self.parser.clear();
        self.set_abort_flag(true);
    }
    fn f_see_all(&mut self) {
        for i in 0..self.dictionary.len() {
            self.word_see(i);
        }
        for (key, index) in self.defined_variables.iter() {
            self.variable_see(key, *index);
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
    fn f_loaded(&mut self) {
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
    }

    fn add_variable(&mut self, name: &str, val: i64) -> usize {
        self.dictionary
            .push(ForthToken::Variable(name.to_owned(), val));
        self.dictionary.len() - 1
    }

    pub fn var_set(&mut self, addr: usize, new_val: i64) {
        // set the variable at addr to val
        let address = addr.max(0) as usize;
        if address < self.dictionary.len() {
            let var = &self.dictionary[addr];
            match var {
                ForthToken::Variable(name, v) => {
                    self.dictionary[addr] = ForthToken::Variable(name.to_owned(), new_val)
                }
                _ => self
                    .msg
                    .error("sysvar_set", "Does not point to variable", Some(addr)),
            }
        }
    }

    pub fn var_get(&mut self, addr: usize) -> i64 {
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
}
