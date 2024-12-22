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
pub struct PrintLogger;

impl Logger for PrintLogger {
    fn header(&self) {
        // Log the header using log crate
        info!("\nTime | Node | Event      | Detail");
    }

    fn log(&self, ident: i32, event: &str, detail: &str) {
        // Log using log crate
        let now = chrono::Local::now().format("%H:%M:%S");
        debug!("{:5} | {:4} | {:10} | {}", now, ident, event, detail);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_logger() {
        // Create a buffer to capture output
        let logger = PrintLogger {};
        logger.header();
        logger.log(1, "TEST", "test detail");
    }
}
