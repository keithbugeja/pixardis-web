#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;

// Global error collector for WASM builds
#[cfg(target_arch = "wasm32")]
static ERROR_COLLECTOR: Mutex<Vec<String>> = Mutex::new(Vec::new());

// Conditional eprintln! macro
#[cfg(target_arch = "wasm32")]
macro_rules! eprintln {
    ($($arg:tt)*) => {
        {
            let message = format!($($arg)*);
            if let Ok(mut errors) = ERROR_COLLECTOR.lock() {
                errors.push(message);
            }
        }
    };
}

// For non-WASM builds, use regular eprintln!
#[cfg(not(target_arch = "wasm32"))]
macro_rules! eprintln {
    ($($arg:tt)*) => {
        std::eprintln!($($arg)*)
    };
}

// Function to get captured errors (WASM only)
#[cfg(target_arch = "wasm32")]
pub fn get_captured_errors() -> Vec<String> {
    ERROR_COLLECTOR.lock().unwrap().drain(..).collect()
}

#[cfg(target_arch = "wasm32")]
pub fn clear_captured_errors() {
    ERROR_COLLECTOR.lock().unwrap().clear();
}

#[derive(Debug, Clone, Copy)]
pub enum LoggerError {
    Lexical,
    Syntax,
    Semantic,
    Type,
    NameResolution,
}

pub enum LoggerMessage {
    Silent,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Logger<'a> {
    source: &'a str,
    newlines: Vec<usize>,        
}

impl<'a> Logger<'a> {
    pub fn new(source: &'a str) -> Logger<'a> {
        let mut logger = Logger {
            source: source,
            newlines: Vec::new(),
        };

        logger.enumerate_newlines();
        logger
    }

    fn enumerate_newlines(&mut self) {
        for (line_end, _) in self.source.match_indices("\n") {
            self.newlines.push(line_end);
        }
    }

    fn get_source_line(&self, line_number: usize) -> &str {
        if line_number == 0 {
            let line_start = 0;
            let line_end = self.newlines[1];
            &self.source[line_start..line_end - 1]
        } else {
            let line_start = self.newlines[line_number - 1];
            let line_end = self.newlines[line_number];
            &self.source[line_start..line_end]
        }
    }

    pub fn print_message(&self, category: LoggerMessage, message: &str) {
        match category {
            LoggerMessage::Silent => eprintln!("{}", message),
            LoggerMessage::Info => eprintln!("Info: {}", message),
            LoggerMessage::Warning => eprintln!("Warning: {}", message),
            LoggerMessage::Error => eprintln!("Error: {}", message),
        }
    }

    pub fn print_short_error(&self, category: LoggerError, message: &str) {
        match category {
            LoggerError::Lexical => eprintln!("Lexical Error: {}\n", message),
            LoggerError::Syntax => eprintln!("Syntax Error: {}\n", message),
            LoggerError::Semantic => eprintln!("Semantic Error: {}\n", message),
            LoggerError::Type => eprintln!("Type Error: {}\n", message),
            LoggerError::NameResolution => eprintln!("Name Resolution Error: {}\n", message),
        }
    }

    pub fn print_error(&self, category: LoggerError, message: &str, line_number: usize) {
        eprintln!("In Line {}: {}", line_number + 1, self.get_source_line(line_number));        
        self.print_short_error(category, message);
    }
}