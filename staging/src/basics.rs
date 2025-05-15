//! Basic Rust concepts and simple interview questions
//!
//! This module covers fundamental Rust concepts that often appear in interviews:
//! - Basic syntax and data types
//! - Ownership and borrowing
//! - Pattern matching
//! - Iterators and closures

/// Problem: Fizz Buzz
///
/// Write a function that returns:
/// - "Fizz" for numbers divisible by 3
/// - "Buzz" for numbers divisible by 5
/// - "FizzBuzz" for numbers divisible by both 3 and 5
/// - The string representation of the number otherwise
pub fn fizz_buzz(n: u32) -> String {
    match (n % 3, n % 5) {
        (0, 0) => "FizzBuzz".to_string(),
        (0, _) => "Fizz".to_string(),
        (_, 0) => "Buzz".to_string(),
        _ => n.to_string(),
    }
}

/// Problem: Reverse a string
///
/// Write a function that reverses a string in place.
/// Note that in Rust, strings are UTF-8 encoded, so we need to be careful with character boundaries.
pub fn reverse_string(s: &str) -> String {
    // Reverse by character (not bytes)
    s.chars().rev().collect()
}

/// Problem: Find the factorial of a number
///
/// Write a function that calculates n! (n factorial).
/// Handle potential overflows appropriately.
pub fn factorial(n: u64) -> Result<u64, &'static str> {
    match n {
        0 | 1 => Ok(1),
        n => {
            let mut result: u64 = 1;
            for i in 2..=n {
                result = result.checked_mul(i).ok_or("Factorial overflow")?;
            }
            Ok(result)
        }
    }
}

/// Problem: Calculate the nth Fibonacci number
///
/// Write a function to calculate the nth number in the Fibonacci sequence.
pub fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0;
            let mut b = 1;
            for _ in 2..=n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        }
    }
}

/// Problem: Implement a generic min function
///
/// Write a function that returns the minimum of two values.
pub fn min<T: Ord>(a: T, b: T) -> T {
    if a <= b { a } else { b }
}

/// Problem: Implement a generic max function
///
/// Write a function that returns the maximum of two values.
pub fn max<T: Ord>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

/// Problem: Check if a string is a palindrome
///
/// Write a function that checks whether a string is a palindrome.
pub fn is_palindrome(s: &str) -> bool {
    // Convert to lowercase and filter out non-alphanumeric characters
    let chars: Vec<char> = s.chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect();

    let n = chars.len();
    for i in 0..n/2 {
        if chars[i] != chars[n-1-i] {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fizz_buzz() {
        assert_eq!(fizz_buzz(1), "1");
        assert_eq!(fizz_buzz(3), "Fizz");
        assert_eq!(fizz_buzz(5), "Buzz");
        assert_eq!(fizz_buzz(15), "FizzBuzz");
        assert_eq!(fizz_buzz(30), "FizzBuzz");
    }

    #[test]
    fn test_reverse_string() {
        assert_eq!(reverse_string("hello"), "olleh");
        assert_eq!(reverse_string("rust"), "tsur");
        // Test with Unicode characters
        assert_eq!(reverse_string("привет"), "тевирп");
        assert_eq!(reverse_string("你好"), "好你");
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), Ok(1));
        assert_eq!(factorial(1), Ok(1));
        assert_eq!(factorial(5), Ok(120));
        assert_eq!(factorial(10), Ok(3628800));
        // This would overflow u64
        assert!(factorial(100).is_err());
    }

    #[test]
    fn test_fibonacci() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(2), 1);
        assert_eq!(fibonacci(3), 2);
        assert_eq!(fibonacci(4), 3);
        assert_eq!(fibonacci(5), 5);
        assert_eq!(fibonacci(10), 55);
    }

    #[test]
    fn test_min_max() {
        assert_eq!(min(5, 10), 5);
        assert_eq!(min(-5, 5), -5);
        assert_eq!(min("abc", "def"), "abc");

        assert_eq!(max(5, 10), 10);
        assert_eq!(max(-5, 5), 5);
        assert_eq!(max("abc", "def"), "def");
    }

    #[test]
    fn test_is_palindrome() {
        assert!(is_palindrome("racecar"));
        assert!(is_palindrome("A man, a plan, a canal: Panama"));
        assert!(is_palindrome("No lemon, no melon"));
        assert!(!is_palindrome("hello"));
        assert!(!is_palindrome("rust"));
    }
}
