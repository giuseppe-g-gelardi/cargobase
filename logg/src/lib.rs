// #[cfg(feature = "logging")]
// use ansi_term::Colour;

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        #[cfg(feature = "logging")]
        println!("{} {}", $crate::color::info_prefix(), format!($($arg)*));
        #[cfg(feature = "tracing")]
        tracing::info!($($arg)*);
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        #[cfg(feature = "logging")]
        println!("{} {}", $crate::color::warn_prefix(), format!($($arg)*));
        #[cfg(feature = "tracing")]
        tracing::warn!($($arg)*);
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        #[cfg(feature = "logging")]
        println!("{} {}", $crate::color::error_prefix(), format!($($arg)*));
        #[cfg(feature = "tracing")]
        tracing::error!($($arg)*);
    }};
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        #[cfg(feature = "logging")]
        println!("{} {}", $crate::color::debug_prefix(), format!($($arg)*));
        #[cfg(feature = "tracing")]
        tracing::debug!($($arg)*);
    }};
}

pub mod color {
    #[cfg(feature = "logging")]
    pub fn info_prefix() -> String {
        ansi_term::Colour::Green.paint("[INFO]").to_string()
    }

    #[cfg(feature = "logging")]
    pub fn warn_prefix() -> String {
        ansi_term::Colour::Yellow.paint("[WARN]").to_string()
    }

    #[cfg(feature = "logging")]
    pub fn error_prefix() -> String {
        ansi_term::Colour::Red.paint("[ERROR]").to_string()
    }

    #[cfg(feature = "logging")]
    pub fn debug_prefix() -> String {
        ansi_term::Colour::Blue.paint("[DEBUG]").to_string()
    }
}
