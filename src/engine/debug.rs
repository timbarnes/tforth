// Debugging help

impl TF {
    fn f_show_stack(&mut self) {
        self.show_stack = true;
    }

    fn f_hide_stack(&mut self) {
        self.show_stack = false;
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

    fn f_see_all(&mut self) {
        for i in 0..self.dictionary.len() {
            self.see_word(i);
        }
    }

    fn f_stack_depth(&mut self) {
        self.stack.push(self.stack.len() as i64);
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
}
