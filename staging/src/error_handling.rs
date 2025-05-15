//! Error Handling Examples and Interview Questions
//!
//! This module demonstrates Rust's error handling patterns:
//! - Working with Result and Option types
//! - Custom error types
//! - Error propagation
//! - Handling multiple error types

use std::fs::File;
use std::io::{self, Read};
use std::num::ParseIntError;
use std::path::Path;
use thiserror::Error;

/// A custom error enum using the thiserror crate
#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseIntError),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unexpected: {0}")]
    Unexpected(String),
}

/// Problem: Read a file and parse its contents as an integer
///
/// This function demonstrates error propagation and conversion.
pub fn read_number_from_file(path: impl AsRef<Path>) -> Result<i32, AppError> {
    let mut file = File::open(path)?;  // io::Error -> AppError via From trait
    let mut content = String::new();
    file.read_to_string(&mut content)?;  // io::Error -> AppError via From trait
    let number = content.trim().parse::<i32>()?;  // ParseIntError -> AppError via From trait
    Ok(number)
}

/// Problem: Safely divide two numbers
///
/// Return an error when attempting to divide by zero.
pub fn safe_divide(a: i32, b: i32) -> Result<i32, AppError> {
    if b == 0 {
        Err(AppError::InvalidInput("Cannot divide by zero".to_string()))
    } else {
        Ok(a / b)
    }
}

/// Problem: Implement a function that attempts multiple strategies
///
/// Try multiple approaches and return the first successful result.
pub fn try_multiple_strategies(input: &str) -> Result<i32, AppError> {
    // Strategy 1: Parse directly
    let strategy1 = input.parse::<i32>();

    // Strategy 2: Parse as hex
    let strategy2 = i32::from_str_radix(input.trim_start_matches("0x"), 16);

    // Strategy 3: Count characters and use as number
    let strategy3 = Ok(input.chars().count() as i32);

    // Try strategies in order
    strategy1
        .or_else(|_| strategy2)
        .or_else(|_| strategy3)
        .map_err(|e| AppError::Parse(e))
}

/// Problem: Map Option to Result
///
/// Convert an Option to a Result with a custom error message.
pub fn option_to_result<T>(opt: Option<T>, error_msg: &str) -> Result<T, AppError> {
    opt.ok_or_else(|| AppError::NotFound(error_msg.to_string()))
}

/// Problem: Find an element in a collection
///
/// Return the element if found, or an error if not.
pub fn find_element<'a, T: PartialEq + std::fmt::Debug>(collection: &'a [T], element: &T) -> Result<&'a T, AppError> {
    collection
        .iter()
        .find(|&x| x == element)
        .ok_or(AppError::NotFound(format!("Element {:?} not found", element)))
}

/// Problem: Parse a complex data structure
///
/// Demonstrates propagating errors from multiple parts of a computation.
pub fn parse_complex_data(data: &str) -> Result<Vec<i32>, AppError> {
    let parts: Vec<&str> = data.split(',').collect();

    if parts.is_empty() {
        return Err(AppError::InvalidInput("Empty data".to_string()));
    }

    parts
        .iter()
        .map(|part| {
            part.trim()
                .parse::<i32>()
                .map_err(AppError::from)
        })
        .collect()
}

/// Problem: Function that can fail in multiple ways
///
/// This function demonstrates handling multiple potential failure points.
pub fn process_user_input(input: &str) -> Result<String, AppError> {
    if input.is_empty() {
        return Err(AppError::InvalidInput("Input cannot be empty".to_string()));
    }

    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(AppError::InvalidInput("Input must have at least two parts".to_string()));
    }

    let command = parts[0].to_lowercase();
    let value = parts[1];

    match command.as_str() {
        "add" => {
            let num = value.parse::<i32>()?;
            Ok(format!("Added {}", num + 10))
        },
        "multiply" => {
            let num = value.parse::<i32>()?;
            Ok(format!("Multiplied {}", num * 10))
        },
        "greet" => {
            Ok(format!("Hello, {}!", value))
        },
        _ => Err(AppError::InvalidInput(format!("Unknown command: {}", command))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_safe_divide() {
        assert_eq!(safe_divide(10, 2), Ok(5));
        assert!(matches!(safe_divide(10, 0), Err(AppError::InvalidInput(_))));
    }

    #[test]
    fn test_try_multiple_strategies() {
        assert_eq!(try_multiple_strategies("42"), Ok(42));
        assert_eq!(try_multiple_strategies("0x2A"), Ok(42));
        assert_eq!(try_multiple_strategies("hello"), Ok(5));
    }

    #[test]
    fn test_option_to_result() {
        let some_value: Option<i32> = Some(42);
        assert_eq!(option_to_result(some_value, "Not found"), Ok(42));

        let none_value: Option<i32> = None;
        assert!(matches!(
            option_to_result(none_value, "Not found"),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn test_find_element() {
        let collection = vec![1, 2, 3, 4, 5];
        assert_eq!(find_element(&collection, &3), Ok(&3));
        assert!(matches!(
            find_element(&collection, &6),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn test_parse_complex_data() {
        assert_eq!(parse_complex_data("1,2,3,4,5"), Ok(vec![1, 2, 3, 4, 5]));
        assert!(matches!(
            parse_complex_data("1,2,a,4,5"),
            Err(AppError::Parse(_))
        ));
        assert!(matches!(
            parse_complex_data(""),
            Err(AppError::InvalidInput(_))
        ));
    }

    #[test]
    fn test_process_user_input() {
        assert_eq!(process_user_input("add 5"), Ok("Added 15".to_string()));
        assert_eq!(process_user_input("multiply 5"), Ok("Multiplied 50".to_string()));
        assert_eq!(process_user_input("greet Alice"), Ok("Hello, Alice!".to_string()));

        assert!(matches!(
            process_user_input(""),
            Err(AppError::InvalidInput(_))
        ));
        assert!(matches!(
            process_user_input("invalid 5"),
            Err(AppError::InvalidInput(_))
        ));
        assert!(matches!(
            process_user_input("add abc"),
            Err(AppError::Parse(_))
        ));
    }

    #[test]
    fn test_read_number_from_file() {
        // Create a temporary file with a number
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "42").unwrap();

        assert_eq!(read_number_from_file(file.path()), Ok(42));

        // Create a file with invalid content
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "not a number").unwrap();

        assert!(matches!(
            read_number_from_file(file.path()),
            Err(AppError::Parse(_))
        ));
    }
}
