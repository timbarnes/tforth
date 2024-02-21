// Compiler and Interpreter

use crate::engine::{FALSE, STACK_START, TF, TRUE};
use crate::tokenizer::is_integer;

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

impl TF {
    pub fn f_lbracket(&mut self) {
        self.set_compile_mode(false);
    }

    pub fn f_rbracket(&mut self) {
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

    pub fn f_execute(&mut self) {
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

    pub fn f_interpret(&mut self) {
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

    pub fn f_number_q(&mut self) {
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

    pub fn f_q_unique(&mut self) {
        // see if a word is unique. Result boolean on stack
        match self.stack.pop() {
            Some(v) => self.stack.push(TRUE),
            None => self.stack.push(FALSE),
        }
    }

    pub fn f_tick(&mut self) {
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

    pub fn f_text(&mut self) {
        // take delimiter from stack; grab string from TIB
        // need to check if TIB is empty
        // if delimiter = 1, get the rest of the TIB
        if stack_ok!(self, 1, "text") {
            let delim = pop!(self) as u8;
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
    }

    pub fn f_colon(&mut self) {
        self.set_compile_mode(true);
    }

    /*  fn f_d_pack(&mut self) {
        // pack the string in PAD and place it in the dictionary for a new word
        let data = self.f_string_at(addr);
        let packed = self.pack_string(&data);
        for c in packed {
            let here = self.data[self.here_ptr];
            self.data[]
        }
    }
    */
}
