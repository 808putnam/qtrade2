//! Rust Interview Questions and Solutions
//!
//! This library contains solutions to common Rust interview questions,
//! organized by topic.

pub mod basics;
pub mod collections;
pub mod error_handling;
pub mod concurrency;
pub mod async_examples;
pub mod algorithms;
pub mod smart_pointers;    // Memory management patterns
pub mod traits_generics;   // Advanced trait usage and generics
pub mod iterators;         // Custom iterators and functional programming
pub mod macros;            // Macro examples (both declarative and procedural)
pub mod testing_patterns;  // Advanced testing techniques
pub mod unsafe_rust;       // Working with unsafe code
pub mod ffi;               // Foreign Function Interface examples
pub mod pattern_matching;  // Advanced pattern matching
pub mod serialization;     // Working with serde
pub mod networking;        // Networking patterns
pub mod blockchain;        // Blockchain development patterns

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}