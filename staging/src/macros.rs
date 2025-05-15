//! Macros Examples
//!
//! This module demonstrates both declarative and procedural macros:
//! - Basic macro_rules! macros
//! - Pattern matching in macros
//! - Hygienic macros
//! - Recursive macros
//! - Macro hygiene and variable capture

/// Problem: Create a simple macro that creates a vector
///
/// Make a macro that takes a list of elements and creates a vec
#[macro_export]
macro_rules! my_vec {
    // Match zero elements
    () => {
        Vec::new()
    };
    // Match one or more elements separated by commas
    ($($element:expr),+ $(,)?) => {
        {
            let mut v = Vec::new();
            $(
                v.push($element);
            )*
            v
        }
    };
}

/// Problem: Create a debug macro that prints file and line information
///
/// Make a macro that prints expressions with source location
#[macro_export]
macro_rules! debug_print {
    ($($arg:expr),*) => {
        println!("[{}:{}] {}", file!(), line!(), format!($($arg),*));
    };
}

/// Problem: Create a macro that performs simple math operations
///
/// Take an operation name and expressions and compute the result
#[macro_export]
macro_rules! math_op {
    (min $($x:expr),+ $(,)?) => {
        {
            let mut min_val = std::i32::MAX;
            $(
                let val = $x;
                if val < min_val {
                    min_val = val;
                }
            )*
            min_val
        }
    };
    (max $($x:expr),+ $(,)?) => {
        {
            let mut max_val = std::i32::MIN;
            $(
                let val = $x;
                if val > max_val {
                    max_val = val;
                }
            )*
            max_val
        }
    };
    (sum $($x:expr),+ $(,)?) => {
        {
            let mut sum = 0;
            $(
                sum += $x;
            )*
            sum
        }
    };
}

/// Problem: Create a macro that generates struct implementations
///
/// Generate getters and setters for struct fields
#[macro_export]
macro_rules! make_getters_setters {
    ($struct_name:ident, $($field_name:ident: $field_type:ty),* $(,)?) => {
        impl $struct_name {
            $(
                pub fn $field_name(&self) -> &$field_type {
                    &self.$field_name
                }

                paste::paste! {
                    pub fn [<set_ $field_name>](&mut self, value: $field_type) {
                        self.$field_name = value;
                    }
                }
            )*
        }
    };
}

/// Problem: Create a recursive macro for nested expressions
///
/// Parse and evaluate nested mathematical expressions
#[macro_export]
macro_rules! nested_expr {
    // Base case: single value
    ($x:expr) => { $x };

    // Recursive case: addition
    (($x:expr) + $($rest:tt)*) => {
        $x + nested_expr!($($rest)*)
    };

    // Recursive case: multiplication
    (($x:expr) * $($rest:tt)*) => {
        $x * nested_expr!($($rest)*)
    };
}

/// Problem: Create a macro that implements a trait for multiple types
///
/// Implement a common trait for a list of types
#[macro_export]
macro_rules! impl_display_for {
    ($($t:ty),* $(,)?) => {
        $(
            impl std::fmt::Display for $t {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
        )*
    };
}

/// Helper macro to demonstrate macro hygiene
#[macro_export]
macro_rules! capture_then_match_tokens {
    ($value:expr) => {
        {
            let value = "original";
            // The macro uses $value, not the local "value"
            assert_ne!($value, value);
            println!("The macro parameter is: {}, not {}", $value, value);
            $value
        }
    };
}

/// Problem: Create a builder pattern with macros
///
/// Generate code for the builder pattern
#[macro_export]
macro_rules! builder {
    // Define a struct and its builder
    (
        $(#[$struct_meta:meta])*
        struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_type:ty,
            )*
        }
    ) => {
        // Define the main struct
        $(#[$struct_meta])*
        struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field_name: $field_type,
            )*
        }

        // Define the builder struct
        paste::paste! {
            pub struct [<$name Builder>] {
                $(
                    $field_name: Option<$field_type>,
                )*
            }

            impl [<$name Builder>] {
                pub fn new() -> Self {
                    Self {
                        $(
                            $field_name: None,
                        )*
                    }
                }

                $(
                    pub fn $field_name(mut self, value: $field_type) -> Self {
                        self.$field_name = Some(value);
                        self
                    }
                )*

                pub fn build(self) -> Result<$name, String> {
                    Ok($name {
                        $(
                            $field_name: self.$field_name.ok_or(
                                format!("Field {} is not set", stringify!($field_name))
                            )?,
                        )*
                    })
                }
            }

            impl $name {
                pub fn builder() -> [<$name Builder>] {
                    [<$name Builder>]::new()
                }
            }
        }
    };
}

// For demonstration purposes, include a few structs with the macros applied
#[derive(Debug, PartialEq)]
pub struct Person {
    name: String,
    age: u32,
}

// We'll use the make_getters_setters macro in tests

#[cfg(test)]
mod tests {
    // Import our macros explicitly for testing
    use crate::{my_vec, math_op, nested_expr, capture_then_match_tokens};
    use super::*;

    // Need paste for macro expansion
    use paste::paste;

    #[test]
    fn test_my_vec() {
        let v1: Vec<i32> = my_vec![];
        assert_eq!(v1, Vec::<i32>::new());

        let v2 = my_vec![1, 2, 3];
        assert_eq!(v2, vec![1, 2, 3]);

        let v3 = my_vec![1, 2, 3,]; // trailing comma
        assert_eq!(v3, vec![1, 2, 3]);
    }

    #[test]
    fn test_math_op() {
        assert_eq!(math_op!(min 5, 3, 8, 1, 4), 1);
        assert_eq!(math_op!(max 5, 3, 8, 1, 4), 8);
        assert_eq!(math_op!(sum 1, 2, 3, 4, 5), 15);
    }

    #[test]
    fn test_nested_expr() {
        assert_eq!(nested_expr!(1), 1);
        assert_eq!(nested_expr!(1 + 2), 3);
        assert_eq!(nested_expr!(1 + 2 + 3), 6);
        assert_eq!(nested_expr!(1 * 2 * 3), 6);
        assert_eq!(nested_expr!(1 + 2 * 3), 7); // No precedence handling, left to right
    }

    #[test]
    fn test_getters_setters() {
        use crate::make_getters_setters;

        #[derive(Debug, PartialEq)]
        struct TestStruct {
            field1: i32,
            field2: String,
        }

        make_getters_setters!(TestStruct, field1: i32, field2: String);

        let mut test = TestStruct {
            field1: 42,
            field2: "hello".to_string(),
        };

        assert_eq!(*test.field1(), 42);
        assert_eq!(*test.field2(), "hello");

        test.set_field1(100);
        test.set_field2("world".to_string());

        assert_eq!(*test.field1(), 100);
        assert_eq!(*test.field2(), "world");
    }

    #[test]
    fn test_hygiene() {
        let result = capture_then_match_tokens!("captured");
        assert_eq!(result, "captured");
    }

    #[test]
    fn test_builder() {
        use crate::builder;

        builder! {
            #[derive(Debug, PartialEq)]
            struct TestUser {
                pub username: String,
                pub email: String,
                pub active: bool,
            }
        }

        let user = TestUser::builder()
            .username("testuser".to_string())
            .email("test@example.com".to_string())
            .active(true)
            .build()
            .unwrap();

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.active, true);

        // Test missing field
        let incomplete_builder = TestUser::builder()
            .username("testuser".to_string())
            .active(true);

        assert!(incomplete_builder.build().is_err());
    }
}
