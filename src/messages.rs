// Message handler

use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum DebugLevel {
    Errors,
    Warnings,
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
            debug_level: DebugLevel::Errors,
        }
    }
    pub fn set_level(&mut self, lev: DebugLevel) {
        self.debug_level = lev;
    }

    pub fn get_level(&self) -> DebugLevel {
        return self.debug_level.clone();
    }

    pub fn debug<T: Debug>(&self, context: &str, text: &str, val: T) {
        match self.debug_level {
            DebugLevel::Debug => {
                println!("DEBUG: {context}: {text}: {:?}", val);
            }
            _ => {
                return;
            }
        }
    }

    pub fn info<T: Debug>(&self, context: &str, text: &str, val: T) {
        match self.debug_level {
            DebugLevel::Info | DebugLevel::Debug => {
                println!("INFO: {context}: {text}: {:?}", val);
            }
            _ => {
                return;
            }
        }
    }

    pub fn warning<T: Debug>(&self, context: &str, text: &str, val: T) {
        match self.debug_level {
            DebugLevel::Warnings | DebugLevel::Info | DebugLevel::Debug => {
                println!("WARNING: {context}: {text}: {:?}", val);
            }
            _ => {
                return;
            }
        }
    }

    pub fn error<T: Debug>(&self, context: &str, text: &str, val: T) {
        println!("ERROR: {context}: {text}: {:?}", val);
    }
}
