//! Testing Patterns Examples
//!
//! This module demonstrates advanced testing techniques:
//! - Property-based testing
//! - Parameterized tests
//! - Snapshot testing
//! - Mocking and test fixtures
//! - Integration testing patterns
//! - Test helpers and utilities

#[cfg(test)]
use proptest::prelude::*;
use std::collections::HashMap;

/// Problem: Implement a function to sort an array
///
/// A simple function to test with various techniques
pub fn sort_array<T: Ord>(mut arr: Vec<T>) -> Vec<T> {
    arr.sort();
    arr
}

/// Problem: Implement a function with specific properties to test
///
/// This function calculates the average of a slice of numbers
pub fn average(numbers: &[f64]) -> Option<f64> {
    if numbers.is_empty() {
        return None;
    }

    let sum: f64 = numbers.iter().sum();
    Some(sum / numbers.len() as f64)
}

/// Problem: Create a function to test with mocks
///
/// A function that uses a dependency that can be mocked
pub trait DataProvider {
    fn get_data(&self) -> Vec<i32>;
}

pub fn process_data<T: DataProvider>(provider: &T) -> i32 {
    let data = provider.get_data();
    data.iter().sum()
}

/// Problem: A function with error handling for testing
///
/// This function has various edge cases to test
pub fn parse_config(config_str: &str) -> Result<HashMap<String, String>, String> {
    let mut config = HashMap::new();

    for line in config_str.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse "key=value" pairs
        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim();
            let value = line[pos+1..].trim();

            if key.is_empty() {
                return Err("Empty key found".to_string());
            }

            config.insert(key.to_string(), value.to_string());
        } else {
            return Err(format!("Invalid line: {}", line));
        }
    }

    Ok(config)
}

/// Problem: A struct to test with fixtures
///
/// This calculator supports basic operations
pub struct Calculator {
    memory: f64,
}

impl Calculator {
    pub fn new() -> Self {
        Calculator { memory: 0.0 }
    }

    pub fn add(&mut self, value: f64) -> f64 {
        self.memory += value;
        self.memory
    }

    pub fn subtract(&mut self, value: f64) -> f64 {
        self.memory -= value;
        self.memory
    }

    pub fn multiply(&mut self, value: f64) -> f64 {
        self.memory *= value;
        self.memory
    }

    pub fn divide(&mut self, value: f64) -> Result<f64, String> {
        if value == 0.0 {
            return Err("Cannot divide by zero".to_string());
        }
        self.memory /= value;
        Ok(self.memory)
    }

    pub fn clear(&mut self) {
        self.memory = 0.0;
    }

    pub fn get_memory(&self) -> f64 {
        self.memory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Basic unit tests

    #[test]
    fn test_sort_array() {
        let arr = vec![3, 1, 4, 1, 5, 9, 2, 6];
        let sorted = sort_array(arr);
        assert_eq!(sorted, vec![1, 1, 2, 3, 4, 5, 6, 9]);
    }

    #[test]
    fn test_average() {
        assert_eq!(average(&[1.0, 2.0, 3.0]), Some(2.0));
        assert_eq!(average(&[]), None);

        let result = average(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        assert!((result - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_config() {
        let config = "
            # This is a comment
            key1=value1
            key2 = value2

            # Another comment
            key3 = value with spaces
        ";

        let result = parse_config(config).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result.get("key1"), Some(&"value1".to_string()));
        assert_eq!(result.get("key2"), Some(&"value2".to_string()));
        assert_eq!(result.get("key3"), Some(&"value with spaces".to_string()));
    }

    #[test]
    fn test_parse_config_errors() {
        // Test with invalid line
        let invalid_config = "key1=value1\ninvalid_line\nkey2=value2";
        assert!(parse_config(invalid_config).is_err());

        // Test with empty key
        let empty_key_config = "key1=value1\n=empty_key\nkey2=value2";
        assert!(parse_config(empty_key_config).is_err());
    }

    // Test with mock objects

    struct MockDataProvider {
        data: Vec<i32>,
    }

    impl DataProvider for MockDataProvider {
        fn get_data(&self) -> Vec<i32> {
            self.data.clone()
        }
    }

    #[test]
    fn test_process_data() {
        let provider = MockDataProvider {
            data: vec![1, 2, 3, 4],
        };

        assert_eq!(process_data(&provider), 10);

        let empty_provider = MockDataProvider {
            data: vec![],
        };

        assert_eq!(process_data(&empty_provider), 0);
    }

    // Test fixtures

    struct CalculatorFixture {
        calc: Calculator,
    }

    impl CalculatorFixture {
        fn new() -> Self {
            CalculatorFixture {
                calc: Calculator::new(),
            }
        }

        fn with_memory(memory: f64) -> Self {
            let mut fixture = Self::new();
            fixture.calc.memory = memory;
            fixture
        }
    }

    #[test]
    fn test_calculator_operations() {
        let mut calc = CalculatorFixture::new().calc;

        assert_eq!(calc.get_memory(), 0.0);

        assert_eq!(calc.add(5.0), 5.0);
        assert_eq!(calc.subtract(2.0), 3.0);
        assert_eq!(calc.multiply(3.0), 9.0);
        assert_eq!(calc.divide(3.0).unwrap(), 3.0);

        calc.clear();
        assert_eq!(calc.get_memory(), 0.0);
    }

    #[test]
    fn test_calculator_divide_by_zero() {
        let mut calc = CalculatorFixture::with_memory(10.0).calc;

        assert!(calc.divide(0.0).is_err());
        // Memory should remain unchanged
        assert_eq!(calc.get_memory(), 10.0);
    }

    // Parameterized tests

    #[test]
    fn test_average_parameterized() {
        let test_cases = vec![
            (vec![1.0, 2.0, 3.0], Some(2.0)),
            (vec![], None),
            (vec![5.0], Some(5.0)),
            (vec![-1.0, 1.0], Some(0.0)),
        ];

        for (input, expected) in test_cases {
            assert_eq!(average(&input), expected);
        }
    }

    // Property-based testing with proptest

    proptest! {
        // The average of identical values should be that value
        #[test]
        fn same_values_have_same_average(x: f64) {
            // Skip NaN and infinity
            prop_assume!(x.is_finite());

            let values = vec![x, x, x];
            prop_assert_eq!(average(&values), Some(x));
        }

        // A sorted array should have the first element as the smallest
        #[test]
        fn sort_finds_minimum(mut vec: Vec<i32>) {
            if vec.is_empty() {
                return;
            }
            let sorted = sort_array(vec.clone());
            let min_value = sorted[0];

            // Check that no value in the original array is less than min_value
            prop_assert!(vec.iter().all(|&x| x >= min_value));
        }

        // The average should always be between the min and max values
        #[test]
        fn average_between_min_and_max(vec: Vec<f64>) {
            // Skip empty arrays and arrays with non-finite values
            prop_assume!(!vec.is_empty());
            prop_assume!(vec.iter().all(|x| x.is_finite()));

            if let Some(avg) = average(&vec) {
                let min = vec.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max = vec.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

                prop_assert!(avg >= min);
                prop_assert!(avg <= max);
            } else {
                prop_assert!(vec.is_empty());
            }
        }
    }
}
