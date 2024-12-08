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

// #[cfg(feature = "logging")]
// use ansi_term::Colour;
//
// #[cfg(feature = "tracing")]
// use tracing;
//
// /// Logs an informational message.
// pub fn info(message: &str) {
//     #[cfg(feature = "tracing")]
//     {
//         tracing::info!("{}", message);
//     }
//
//     #[cfg(feature = "logging")]
//     {
//         println!("{} {}", Colour::Green.paint("[INFO]"), message);
//     }
// }
//
// /// Logs a warning message.
// pub fn warn(message: &str) {
//     #[cfg(feature = "tracing")]
//     {
//         tracing::warn!("{}", message);
//     }
//
//     #[cfg(feature = "logging")]
//     {
//         println!("{} {}", Colour::Yellow.paint("[WARN]"), message);
//     }
// }
//
// /// Logs an error message.
// pub fn error(message: &str) {
//     #[cfg(feature = "tracing")]
//     {
//         tracing::error!("{}", message);
//     }
//
//     #[cfg(feature = "logging")]
//     {
//         println!("{} {}", Colour::Red.paint("[ERROR]"), message);
//     }
// }
//
// /// Logs a debug message.
// pub fn debug(message: &str) {
//     #[cfg(feature = "tracing")]
//     {
//         tracing::debug!("{}", message);
//     }
//
//     #[cfg(feature = "logging")]
//     {
//         println!("{} {}", Colour::Blue.paint("[DEBUG]"), message);
//     }
// }
//
// // #[macro_export]
// // macro_rules! info {
// //     ($($arg:tt)*) => {{
// //         #[cfg(feature = "logging")]
// //         println!("{} {}", $crate::color::info_prefix(), format!($($arg)*));
// //         #[cfg(feature = "tracing")]
// //         tracing::info!($($arg)*);
// //     }};
// // }
// //
// // #[macro_export]
// // macro_rules! warn {
// //     ($($arg:tt)*) => {{
// //         #[cfg(feature = "logging")]
// //         println!("{} {}", $crate::color::warn_prefix(), format!($($arg)*));
// //         #[cfg(feature = "tracing")]
// //         tracing::warn!($($arg)*);
// //     }};
// // }
// //
// // // #[macro_export]
// // // macro_rules! error {
// // //     ($($arg:tt)*) => {{
// // //         #[cfg(feature = "logging")]
// // //         println!("{} {}", $crate::color::error_prefix(), format!($($arg)*));
// // //         #[cfg(feature = "tracing")]
// // //         tracing::error!($($arg)*);
// // //     }};
// // // }
// //
// // #[macro_export]
// // macro_rules! error {
// //     ($($arg:tt)*) => {
// //         {
// //             #[cfg(feature = "tracing")]
// //             {
// //                 tracing::error!($($arg)*);
// //             }
// //             #[cfg(feature = "logging")]
// //             {
// //                 println!("\x1b[31m[ERROR]\x1b[0m {}", format!($($arg)*));
// //             }
// //         }
// //     };
// // }
// //
// // #[macro_export]
// // macro_rules! debug {
// //     ($($arg:tt)*) => {{
// //         #[cfg(feature = "logging")]
// //         println!("{} {}", $crate::color::debug_prefix(), format!($($arg)*));
// //         #[cfg(feature = "tracing")]
// //         tracing::debug!($($arg)*);
// //     }};
// // }
//
// pub mod color {
//     #[cfg(feature = "logging")]
//     pub fn info_prefix() -> String {
//         ansi_term::Colour::Green.paint("[INFO]").to_string()
//     }
//
//     #[cfg(feature = "logging")]
//     pub fn warn_prefix() -> String {
//         ansi_term::Colour::Yellow.paint("[WARN]").to_string()
//     }
//
//     #[cfg(feature = "logging")]
//     pub fn error_prefix() -> String {
//         ansi_term::Colour::Red.paint("[ERROR]").to_string()
//     }
//
//     #[cfg(feature = "logging")]
//     pub fn debug_prefix() -> String {
//         ansi_term::Colour::Blue.paint("[DEBUG]").to_string()
