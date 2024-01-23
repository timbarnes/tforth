#[derive(Debug, Clone)]
pub enum DebugLevel {
    No,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Msg {
    debug_level: DebugLevel,
}

impl Msg {
    pub fn new() -> Msg {
        Msg {
            debug_level: DebugLevel::No,
        }
    }
    pub fn set_level(&mut self, lev: DebugLevel) {
        self.debug_level = lev;
    }

    pub fn get_level(&self) -> DebugLevel {
        return self.debug_level.clone();
    }

    pub fn info(&self, context: &str, text: String) {
        match self.debug_level {
            DebugLevel::Info => {
                println!("INFO:{context}: {text}");
            }
            _ => {
                return;
            }
        }
    }

    pub fn warning(&self, context: &str, text: String) {
        match self.debug_level {
            DebugLevel::Warning | DebugLevel::Info => {
                println!("WARNING:{context}: {text}");
            }
            _ => {
                return;
            }
        }
    }
}
