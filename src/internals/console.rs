/// Input-output words
use crate::engine::{FileMode, BUF_SIZE, STACK_START, TF};
use std::cmp::min;
use std::io::{self, Write};

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
macro_rules! pop1_push1 {
    // Helper macro
    ($self:ident, $word:expr, $expression:expr) => {
        if let Some(x) = $self.pop_one(&$word) {
            $self.stack.push($expression(x));
        }
    };
}
macro_rules! pop1 {
    ($self:ident, $word:expr, $code:expr) => {{
        if let Some(x) = $self.pop_one(&$word) {
            $code(x)
        }
    }};
}

impl TF {
    /// macros:
    ///
    /// pop! attempts to take one element off the computation stack,
    ///      calling abort if underflow
    ///

    pub fn f_key(&mut self) {
        // get a character and push on the stack
        let c = self.parser.reader.read_char();
        match c {
            Some(c) => self.stack.push(c as i64),
            None => self
                .msg
                .error("KEY", "unable to get char from input stream", None::<bool>),
        }
    }

    /// ( b u -- b u ) ACCEPT
    ///
    /// Read up to u characters, storing them at string address b.
    /// Return the start of the string, and the number of characters read.
    ///
    pub fn f_accept(&mut self) {
        if stack_ok!(self, 2, "accept") {
            let dest = pop!(self) as usize;
            let max_len = top!(self);
            match self.parser.reader.get_line(&"".to_owned(), false) {
                Some(line) => {
                    let length = min(line.len() - 1, max_len as usize) as usize;
                    let line_str = &line[..length];
                    let length = line.len();
                    self.u_save_string(line_str, dest, length); // write a counted string
                    push!(self, length as i64);
                }
                None => {
                    self.msg
                        .error("ACCEPT", "Unable to read from input", None::<bool>);
                    self.f_abort();
                }
            }
        }
    }

    pub fn f_query(&mut self) {
        push!(self, self.tib_ptr as i64);
        push!(self, BUF_SIZE as i64);
        self.f_accept();
    }

    // output

    pub fn i_emit(&mut self) {
        if stack_ok!(self, 1, "emit") {
            let c = pop!(self);
            if (0x20..=0x7f).contains(&c) {
                print!("{}", c as u8 as char);
            } else {
                self.msg.error("EMIT", "Arg out of range", Some(c));
            }
        }
    }

    pub fn f_emit(&mut self) {
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

    pub fn f_flush(&mut self) {
        io::stdout().flush().unwrap();
    }

    pub fn f_dot(&mut self) {
        pop1!(self, ".", |a| print!("{a} "));
    }

    pub fn f_dot_s(&mut self) {
        println!("{:?}", self.stack);
    }

    pub fn f_cr(&mut self) {
        println!("");
    }

    /// s" ( -- ) get a string and place it in PAD
    pub fn f_s_quote(&mut self) {
        push!(self, '"' as i64);
        self.f_parse();
    }

    pub fn f_dot_s_quote(&mut self) {
        print!("{:?}", self.get_string(self.pad_ptr));
    }

    /// TYPE - print a string, using the string address on the stack
    pub fn f_type(&mut self) {
        if stack_ok!(self, 1, "type") {
            let addr = pop!(self) as usize;
            let text = self.u_get_string(addr);
            print!("{text}");
        }
    }

    // file i/o

    pub fn f_r_w(&mut self) {
        self.file_mode = FileMode::ReadWrite;
    }
    pub fn f_r_o(&mut self) {
        self.file_mode = FileMode::ReadOnly;
    }
    pub fn f_include_file(&mut self) {
        self.loaded();
    }
}
