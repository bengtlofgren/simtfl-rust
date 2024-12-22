use log::{debug, info};

/// A trait for loggers that defines the basic logging interface
pub trait Logger: Send + Sync {
    /// Print a header (if applicable)
    fn header(&self);

    /// Log an event
    fn log(&self, ident: i32, event: &str, detail: &str);
}

/// A logger that does nothing
#[derive(Default)]
pub struct DebugLogger;

impl Logger for DebugLogger {
    fn header(&self) {
        // Log the header using log crate
        info!(" Node | Event      | Detail");
    }

    fn log(&self, ident: i32, event: &str, detail: &str) {
        // Log using log crate
        debug!(" {:4} | {:10} | {}", ident, event, detail);
    }
}

#[derive(Default)]
pub struct PrintLogger;

impl Logger for PrintLogger {
    fn header(&self) {
        // Log the header using stdout
        println!(" Node | Event      | Detail");
    }

    fn log(&self, ident: i32, event: &str, detail: &str) {
        // Log using stdout
        println!("{:4} | {:10} | {}", ident, event, detail);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_logger() {
        // Create a buffer to capture output
        let logger = DebugLogger {};
        logger.header();
        logger.log(1, "TEST", "test detail");
    }

    #[test]
    fn test_print_logger() {
        // Create a buffer to capture output
        let logger = PrintLogger {};
        logger.header();
        logger.log(1, "TEST", "test detail");
    }
}
