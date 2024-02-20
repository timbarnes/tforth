// General-purpose builtin words

impl TF {
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

    fn f_clear(&mut self) {
        self.stack.clear()
    }

    fn f_bye(&mut self) {
        self.set_exit_flag();
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
}
