// #[cfg(feature = "tracing")]
// use tracing::Level;
//
// #[cfg(feature = "logging")]
// use ansi_term::Colour;

/// Logs an error message.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        #[cfg(feature = "tracing")]
        tracing::event!(Level::ERROR, $($arg)*);

        #[cfg(feature = "logging")]
        println!("{}", ansi_term::Colour::Red.paint(format!($($arg)*)));

        #[cfg(not(any(feature = "logging", feature = "tracing")))]
        let _ = format!($($arg)*); // No-op
    }};
}

/// Logs a warning message.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        #[cfg(feature = "tracing")]
        tracing::event!(Level::WARN, $($arg)*);

        #[cfg(feature = "logging")]
        println!("{}", ansi_term::Colour::Yellow.paint(format!($($arg)*)));

        #[cfg(not(any(feature = "logging", feature = "tracing")))]
        let _ = format!($($arg)*); // No-op
    }};
}

/// Logs an info message.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        #[cfg(feature = "tracing")]
        tracing::event!(Level::INFO, $($arg)*);

        #[cfg(feature = "logging")]
        println!("{}", ansi_term::Colour::Green.paint(format!($($arg)*)));

        #[cfg(not(any(feature = "logging", feature = "tracing")))]
        let _ = format!($($arg)*); // No-op
    }};
}

/// Logs a debug message.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        #[cfg(feature = "tracing")]
        tracing::event!(Level::DEBUG, $($arg)*);

        #[cfg(feature = "logging")]
        println!("{}", ansi_term::Colour::Blue.paint(format!($($arg)*)));

        #[cfg(not(any(feature = "logging", feature = "tracing")))]
        let _ = format!($($arg)*); // No-op
    }};
}

