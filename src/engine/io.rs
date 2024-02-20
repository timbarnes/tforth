/// Input-output words

impl TF {
    /// input

    fn f_key(&mut self) {
        // get a character and push on the stack
        let c = self.parser.reader.read_char();
        match c {
            Some(c) => self.stack.push(c as i64),
            None => self
                .msg
                .error("KEY", "unable to get char from input stream", None::<bool>),
        }
    }
    fn f_accept(&mut self) {
        // get a new line of input and initialize the pointer variables
        // TIB is an unpacked (Rust) string
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

    // output

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

    fn f_dot(&mut self) {
        pop1!(self, ".", |a| print!("{a} "));
    }

    fn f_dot_s(&mut self) {
        println!("{:?}", self.stack);
    }

    fn f_cr(&mut self) {
        println!("");
    }

    fn f_dot_s_quote(&mut self) {
        print!("{:?}", self.get_string_var(self.pad_ptr));
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

    // file i/o

    fn f_r_w(&mut self) {
        self.file_mode = FileMode::ReadWrite;
    }
    fn f_r_o(&mut self) {
        self.file_mode = FileMode::ReadOnly;
    }
    fn f_include_file(&mut self) {
        self.loaded();
    }
}
