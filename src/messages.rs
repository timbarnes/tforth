// Message handler

use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum DebugLevel {
    Error,
    Warning,
    Info,
    Debug,
}

#[derive(Debug, Clone)]
pub struct Msg {
    debug_level: DebugLevel,
}

impl Msg {
    pub fn new() -> Msg {
        Msg {
            debug_level: DebugLevel::Error,
        }
    }
    pub fn set_level(&mut self, lev: DebugLevel) {
        self.debug_level = lev;
    }

    pub fn get_level(&self) -> DebugLevel {
        self.debug_level.clone()
    }

    pub fn debug<T: Debug>(&self, context: &str, text: &str, val: Option<T>) {
        if let DebugLevel::Debug = self.debug_level {
            match val {
                Some(val) => println!("DEBUG: {context}: {text}: {:?}", val),
                None => println!("DEBUG: {context}: {text}"),
            }
        }
    }

    pub fn info<T: Debug>(&self, context: &str, text: &str, val: Option<T>) {
        match self.debug_level {
            DebugLevel::Info | DebugLevel::Debug => match val {
                Some(val) => println!("INFO: {context}: {text}: {:?}", val),
                None => println!("INFO: {context}: {text}"),
            },
            _ => {}
        }
    }

    pub fn warning<T: Debug>(&self, context: &str, text: &str, val: Option<T>) {
        match self.debug_level {
            DebugLevel::Warning | DebugLevel::Info | DebugLevel::Debug => match val {
                Some(val) => println!("WARNING: {context}: {text}: {:?}", val),
                None => println!("WARNING: {context}: {text}"),
            },
            _ => {}
        }
    }

    pub fn error<T: Debug>(&self, context: &str, text: &str, val: Option<T>) {
        match val {
            Some(val) => println!("ERROR: {context}: {text}: {:?}", val),
            None => println!("ERROR: {context}: {text}"),
        }
    }
}
