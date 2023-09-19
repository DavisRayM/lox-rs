//! This library contains utilities for analyzing lox syntax
//!
//! # Examples
//!
//! [Scanner](Scanner) can be used to retrieve valid tokens from a source
//! ```rust
//!
//! ```
pub mod parser;
pub mod scanner;

pub use parser::Parser;
pub use scanner::Scanner;
