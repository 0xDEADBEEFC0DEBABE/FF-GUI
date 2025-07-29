// Logging macros with timestamp and log level
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        println!("[{}][DEBUG] {}", chrono::Local::now().format("%H:%M:%S"), format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        println!("[{}][INFO] {}", chrono::Local::now().format("%H:%M:%S"), format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        println!("[{}][WARN] {}", chrono::Local::now().format("%H:%M:%S"), format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        eprintln!("[{}][ERROR] {}", chrono::Local::now().format("%H:%M:%S"), format!($($arg)*))
    };
}