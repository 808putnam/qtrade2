//! Traits and Generics Examples
//!
//! This module demonstrates advanced Rust traits and generic programming:
//! - Trait bounds
//! - Associated types
//! - Generic implementations
//! - Trait objects
//! - Trait inheritance
//! - Default type parameters
//! - Generic constraints

/// Problem: Implement a generic function with trait bounds
///
/// Create a function that works with any type that can be converted to a string
pub fn to_debug_string<T: std::fmt::Debug>(value: &T) -> String {
    format!("{:?}", value)
}

/// Problem: Create a trait with associated types
///
/// Define a trait for collections with an associated item type
pub trait Container {
    type Item;

    fn add(&mut self, item: Self::Item);
    fn get(&self, index: usize) -> Option<&Self::Item>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Implementation of the Container trait for Vec
impl<T> Container for Vec<T> {
    type Item = T;

    fn add(&mut self, item: Self::Item) {
        self.push(item);
    }

    fn get(&self, index: usize) -> Option<&Self::Item> {
        self.get(index)
    }

    fn len(&self) -> usize {
        self.len()
    }
}

/// Problem: Implement multiple traits with a where clause
///
/// Create a function that requires multiple trait bounds using a where clause
pub fn process_data<T>(data: &[T]) -> String
where
    T: std::fmt::Display + Clone + PartialOrd,
{
    if data.is_empty() {
        return String::from("Empty data");
    }

    let mut largest = &data[0];
    for item in data {
        if item > largest {
            largest = item;
        }
    }

    format!("Largest item: {}", largest)
}

/// Problem: Create a trait with default type parameters
///
/// Demonstrate how to use default type parameters in traits
pub trait Converter<T, U = String> {
    fn convert(&self, item: T) -> U;
}

struct StringConverter;

impl Converter<i32> for StringConverter {
    fn convert(&self, item: i32) -> String {
        item.to_string()
    }
}

impl Converter<String, Vec<u8>> for StringConverter {
    fn convert(&self, item: String) -> Vec<u8> {
        item.into_bytes()
    }
}

/// Problem: Use trait objects for dynamic dispatch
///
/// Create a function that can process different types at runtime
pub fn process_shapes(shapes: Vec<Box<dyn Shape>>) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

pub trait Shape {
    fn area(&self) -> f64;
}

pub struct Circle {
    pub radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

/// Problem: Implement trait inheritance
///
/// Create a trait that inherits from another trait
pub trait Animal {
    fn name(&self) -> &str;
    fn noise(&self) -> &str;
}

pub trait Pet: Animal {
    fn owner(&self) -> &str;
}

pub struct Dog {
    name: String,
    owner: String,
}

impl Animal for Dog {
    fn name(&self) -> &str {
        &self.name
    }

    fn noise(&self) -> &str {
        "Woof!"
    }
}

impl Pet for Dog {
    fn owner(&self) -> &str {
        &self.owner
    }
}

/// Problem: Create a generic type with constraints
///
/// Implement a wrapper that requires its wrapped type to have certain traits
pub struct Wrapper<T: Clone + std::fmt::Debug> {
    value: T,
}

impl<T: Clone + std::fmt::Debug> Wrapper<T> {
    pub fn new(value: T) -> Self {
        Wrapper { value }
    }

    pub fn get(&self) -> T {
        self.value.clone()
    }

    pub fn describe(&self) -> String {
        format!("{:?}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_debug_string() {
        assert_eq!(to_debug_string(&42), "42");
        assert_eq!(to_debug_string(&vec![1, 2, 3]), "[1, 2, 3]");
    }

    #[test]
    fn test_container() {
        let mut container = Vec::new();
        container.add(1);
        container.add(2);

        assert_eq!(container.get(0), Some(&1));
        assert_eq!(container.len(), 2);
        assert!(!container.is_empty());
    }

    #[test]
    fn test_process_data() {
        let data = vec![5, 1, 9, 3, 7];
        assert_eq!(process_data(&data), "Largest item: 9");

        let empty: Vec<i32> = vec![];
        assert_eq!(process_data(&empty), "Empty data");
    }

    #[test]
    fn test_converter() {
        let converter = StringConverter;
        assert_eq!(converter.convert(42), "42");
        assert_eq!(converter.convert(String::from("hello")), b"hello");
    }

    #[test]
    fn test_shapes() {
        let shapes: Vec<Box<dyn Shape>> = vec![
            Box::new(Circle { radius: 1.0 }),
            Box::new(Rectangle { width: 2.0, height: 3.0 }),
        ];

        assert_eq!(process_shapes(shapes), std::f64::consts::PI + 6.0);
    }

    #[test]
    fn test_pet() {
        let dog = Dog {
            name: String::from("Rex"),
            owner: String::from("John"),
        };

        assert_eq!(dog.name(), "Rex");
        assert_eq!(dog.noise(), "Woof!");
        assert_eq!(dog.owner(), "John");
    }

    #[test]
    fn test_wrapper() {
        let wrapper = Wrapper::new(42);
        assert_eq!(wrapper.get(), 42);
        assert_eq!(wrapper.describe(), "42");
    }
}
