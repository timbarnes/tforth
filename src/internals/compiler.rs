// Compiler and Interpreter

use crate::engine::{ADDRESS_MASK, FALSE, IMMEDIATE_MASK, STACK_START, TF, TRUE};
use crate::tokenizer::u_is_integer;

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
    /// immediate - sets the immediate flag on the most recently defined word
    /// Context pointer links to the most recent name field
    pub fn f_immediate(&mut self) {
        let mut mask = self.data[self.context_ptr] as usize;
        mask |= IMMEDIATE_MASK;
        self.data[self.context_ptr] = mask as i64;
    }

    /// [  Install $INTERPRET in 'EVAL
    pub fn f_lbracket(&mut self) {
        self.set_compile_mode(false);
    }

    /// ]  Install $COMPILE in 'EVAL   
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

    /// number? ( a -- n T | a F ) tests a string to see if it's a number;
    /// leaves n and flag on the stack: true if number is ok.
    pub fn f_number_q(&mut self) {
        let buf_addr = pop!(self);
        let mut result = 0;
        let mut flag = 0;
        let numtext = self.u_get_string_var(buf_addr as usize);
        if u_is_integer(&numtext.as_str()) {
            result = numtext.parse().unwrap();
            flag = -1; // valid number
            push!(self, result);
            push!(self, TRUE);
        } else {
            push!(self, buf_addr);
            push!(self, FALSE);
        }
    }

    pub fn f_q_unique(&mut self) {
        // see if a word is unique. Result boolean on stack
        match self.stack.pop() {
            Some(v) => self.stack.push(TRUE),
            None => self.stack.push(FALSE),
        }
    }

    /// looks for a (postfix) word in the dictionary
    /// places it's execution token / address on the stack
    /// builtin addresses have been bumped up by 1000 to distinguish them
    /// Pushes 0 if not found
    pub fn f_tick(&mut self) {
        self.f_s_quote(); // gets a string and places it in PAD
                          // search the dictionary
        let token = self.get_string_var(self.pad_ptr);
        match self.find(&token) {
            Some(idx) => self.stack.push(idx as i64),
            None => self.stack.push(0),
        }
    }

    /// (parse) - b u c -- b u delta )
    /// Find a c-delimited token in the string buffer at b, buffer len u.
    /// Return the pointer to the buffer, the length of the token,
    /// and the offset from the start of the buffer to the start of the token.
    pub fn f_parse_p(&mut self) {
        if stack_ok!(self, 3, "(parse)") {
            let delim = pop!(self) as u8 as char;
            let buf_len = pop!(self);
            let in_p = pop!(self);
            // traverse the string, dropping leading delim characters
            // in_p points *into* a string, so no count field
            let start = in_p as usize + 1;
            let end = start + buf_len as usize;
            let mut i = start as usize;
            let mut j = i;
            while self.strings[i] == delim && i < end {
                i += 1;
            }
            j = i;
            while j < end && self.strings[j] != delim {
                j += 1;
            }
            push!(self, in_p);
            push!(self, (j - i) as i64);
            push!(self, i as i64);
        }
    }

    /// TEXT ( -- ) Get a space-delimited token from the TIB, place in PAD
    pub fn f_text(&mut self) {
        push!(self, ' ' as u8 as i64);
        self.f_parse();
    }

    /// PARSE ( c -- b u ) Get a c-delimited token from TIB, and return counted string in PAD
    /// need to check if TIB is empty
    /// if delimiter = 1, get the rest of the TIB
    pub fn f_parse(&mut self) {
        if stack_ok!(self, 1, "parse") {
            push!(self, (self.tib_in_ptr + self.tib_in_ptr) as i64);
            push!(self, (self.tib_size_ptr - self.tib_in_ptr) as i64);
            push!(self, ' ' as u8 as i64);
            self.f_parse_p();
            // check length, and copy to PAD if a token was found
            let delta = pop!(self);
            let length = pop!(self);
            let addr = pop!(self);
            if length > 0 {
                // copy to pad
                self.u_str_copy((addr + delta) as usize, self.pad_ptr, length as usize);
            }
            self.tib_in_ptr += length as usize;
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
    /// Return a string slice from a Forth string address
    fn u_get_string_var(&mut self, addr: usize) -> String {
        let str_addr = addr & ADDRESS_MASK;
        let last = str_addr + self.strings[str_addr] as usize;
        let mut result = String::new();
        for i in str_addr..last {
            result.push(self.strings[i] as char);
        }
        result
    }

    /// copy a string from a text buffer to a new counted string
    /// Typically used to copy to PAD from TIB
    fn u_str_copy(&mut self, from: usize, to: usize, length: usize) {
        self.strings[to] = length as u8 as char; // count byte
        for i in 0..(length - 1) {
            self.strings[to + i] = self.strings[from + i];
        }
    }
}
