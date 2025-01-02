// Default case without an external printer

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        println!("[{}] {}", "+".bright_green(), format!($($arg)*));
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("[{}] {}", "-".red(), format!($($arg)*));
    };
}

// Case with an external printer

#[macro_export]
macro_rules! ext_success {
    ($printer:expr, $($arg:tt)*) => {
        $printer.print(format!("[{}] {}", "+".bright_green(), format!($($arg)*)).to_string()).unwrap();
    };
}
#[macro_export]
macro_rules! ext {
    ($printer:expr, $($arg:tt)*) => {
        $printer.print(format!("[{}] {}", "-".red(), format!($($arg)*)).to_string()).unwrap();
    };
}