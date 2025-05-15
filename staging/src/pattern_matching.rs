//! Pattern Matching Examples
//!
//! This module demonstrates advanced pattern matching in Rust:
//! - Destructuring complex structures
//! - Match guards
//! - @ bindings
//! - Nested patterns
//! - Ranges in patterns
//! - Rest patterns
//! - Pattern matching in different contexts (let, if let, while let)

/// Problem: Match on complex enums
///
/// Use pattern matching to extract values from complex enums
#[derive(Debug)]
pub enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
    SetUser { name: String, age: u32 },
}

pub fn handle_message(msg: Message) -> String {
    match msg {
        Message::Quit => String::from("Quitting"),
        Message::Move { x, y } => format!("Moving to ({}, {})", x, y),
        Message::Write(text) => format!("Writing: {}", text),
        Message::ChangeColor(r, g, b) => format!("Changing color to ({}, {}, {})", r, g, b),
        Message::SetUser { name, age } => format!("Setting user {} ({})", name, age),
    }
}

/// Problem: Use match guards
///
/// Combine pattern matching with conditional logic
pub fn categorize_number(num: i32) -> &'static str {
    match num {
        n if n < 0 => "negative",
        0 => "zero",
        n if n % 2 == 0 => "positive even",
        _ => "positive odd",
    }
}

/// Problem: Use @ bindings
///
/// Bind values while pattern matching
pub fn describe_number(num: i32) -> String {
    match num {
        n @ 0..=9 => format!("Single digit: {}", n),
        n @ 10..=99 => format!("Double digit: {}", n),
        n @ 100..=999 => format!("Triple digit: {}", n),
        n @ _ if n < 0 => format!("Negative number: {}", n),
        n => format!("Large number: {}", n),
    }
}

/// Problem: Destructure nested structures
///
/// Match on complex, nested data structures
#[derive(Debug)]
pub struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
pub enum Shape {
    Circle(Point, i32),
    Rectangle(Point, Point),
    Triangle(Point, Point, Point),
}

pub fn describe_shape(shape: &Shape) -> String {
    match shape {
        Shape::Circle(Point { x, y }, radius) => {
            format!("Circle at ({}, {}) with radius {}", x, y, radius)
        }
        Shape::Rectangle(
            Point { x: x1, y: y1 },
            Point { x: x2, y: y2 },
        ) => {
            format!("Rectangle from ({}, {}) to ({}, {})", x1, y1, x2, y2)
        }
        Shape::Triangle(
            Point { x: x1, y: y1 },
            Point { x: x2, y: y2 },
            Point { x: x3, y: y3 },
        ) => {
            format!("Triangle at ({}, {}), ({}, {}), ({}, {})",
                x1, y1, x2, y2, x3, y3)
        }
    }
}

/// Problem: Use patterns with multiple possibilities
///
/// Match on multiple options at once
pub fn day_type(day: &str) -> &'static str {
    match day {
        "Monday" | "Tuesday" | "Wednesday" | "Thursday" | "Friday" => "weekday",
        "Saturday" | "Sunday" => "weekend",
        _ => "invalid day",
    }
}

/// Problem: Use if let for selective patterns
///
/// Process only certain patterns without exhaustive matching
pub fn process_option(val: Option<i32>) -> String {
    // Using if let
    if let Some(x) = val {
        if x > 0 {
            format!("Positive: {}", x)
        } else if x < 0 {
            format!("Negative: {}", x)
        } else {
            String::from("Zero")
        }
    } else {
        String::from("No value")
    }
}

/// Problem: Use while let for sequence processing
///
/// Process items from an iterator using pattern matching
pub fn sum_until_negative(mut iter: impl Iterator<Item = i32>) -> i32 {
    let mut sum = 0;

    // Process values until we hit a negative or None
    while let Some(val) = iter.next() {
        if val < 0 {
            break;
        }
        sum += val;
    }

    sum
}

/// Problem: Destructure arrays and slices
///
/// Match on arrays and slices with different patterns
pub fn analyze_sequence(seq: &[i32]) -> String {
    match seq {
        [] => String::from("Empty sequence"),
        [single] => format!("Single element: {}", single),
        [first, second] => format!("Two elements: {} and {}", first, second),
        [first, .., last] => format!("Multiple elements from {} to {}", first, last),
    }
}

/// Problem: Use tuples in patterns
///
/// Match on tuples with multiple elements
pub fn classify_point(point: (i32, i32)) -> &'static str {
    match point {
        (0, 0) => "at origin",
        (0, _) => "on y-axis",
        (_, 0) => "on x-axis",
        (x, y) if x == y => "on y=x line",
        (x, y) if x == -y => "on y=-x line",
        (x, y) if x > 0 && y > 0 => "in first quadrant",
        (x, y) if x < 0 && y > 0 => "in second quadrant",
        (x, y) if x < 0 && y < 0 => "in third quadrant",
        (_, _) => "in fourth quadrant",
    }
}

/// Problem: Use patterns in function parameters
///
/// Apply pattern matching directly in function signatures
pub fn print_coordinates(&(x, y): &(i32, i32)) -> String {
    format!("Coordinates: ({}, {})", x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_message() {
        assert_eq!(handle_message(Message::Quit), "Quitting");
        assert_eq!(
            handle_message(Message::Move { x: 10, y: 20 }),
            "Moving to (10, 20)"
        );
        assert_eq!(
            handle_message(Message::Write(String::from("hello"))),
            "Writing: hello"
        );
        assert_eq!(
            handle_message(Message::ChangeColor(255, 0, 0)),
            "Changing color to (255, 0, 0)"
        );
        assert_eq!(
            handle_message(Message::SetUser {
                name: String::from("John"),
                age: 30,
            }),
            "Setting user John (30)"
        );
    }

    #[test]
    fn test_categorize_number() {
        assert_eq!(categorize_number(-10), "negative");
        assert_eq!(categorize_number(0), "zero");
        assert_eq!(categorize_number(2), "positive even");
        assert_eq!(categorize_number(3), "positive odd");
    }

    #[test]
    fn test_describe_number() {
        assert_eq!(describe_number(5), "Single digit: 5");
        assert_eq!(describe_number(42), "Double digit: 42");
        assert_eq!(describe_number(100), "Triple digit: 100");
        assert_eq!(describe_number(-5), "Negative number: -5");
        assert_eq!(describe_number(1000), "Large number: 1000");
    }

    #[test]
    fn test_describe_shape() {
        let circle = Shape::Circle(Point { x: 0, y: 0 }, 5);
        assert_eq!(
            describe_shape(&circle),
            "Circle at (0, 0) with radius 5"
        );

        let rect = Shape::Rectangle(
            Point { x: 0, y: 0 },
            Point { x: 10, y: 20 },
        );
        assert_eq!(
            describe_shape(&rect),
            "Rectangle from (0, 0) to (10, 20)"
        );

        let triangle = Shape::Triangle(
            Point { x: 0, y: 0 },
            Point { x: 0, y: 10 },
            Point { x: 10, y: 0 },
        );
        assert_eq!(
            describe_shape(&triangle),
            "Triangle at (0, 0), (0, 10), (10, 0)"
        );
    }

    #[test]
    fn test_day_type() {
        assert_eq!(day_type("Monday"), "weekday");
        assert_eq!(day_type("Saturday"), "weekend");
        assert_eq!(day_type("NotADay"), "invalid day");
    }

    #[test]
    fn test_process_option() {
        assert_eq!(process_option(Some(5)), "Positive: 5");
        assert_eq!(process_option(Some(-3)), "Negative: -3");
        assert_eq!(process_option(Some(0)), "Zero");
        assert_eq!(process_option(None), "No value");
    }

    #[test]
    fn test_sum_until_negative() {
        let values = vec![1, 2, 3, -1, 4, 5];
        assert_eq!(sum_until_negative(values.into_iter()), 6);

        let all_positive = vec![1, 2, 3, 4, 5];
        assert_eq!(sum_until_negative(all_positive.into_iter()), 15);
    }

    #[test]
    fn test_analyze_sequence() {
        let empty: Vec<i32> = vec![];
        assert_eq!(analyze_sequence(&empty), "Empty sequence");

        assert_eq!(analyze_sequence(&[42]), "Single element: 42");

        assert_eq!(analyze_sequence(&[1, 2]), "Two elements: 1 and 2");

        assert_eq!(
            analyze_sequence(&[10, 20, 30, 40, 50]),
            "Multiple elements from 10 to 50"
        );
    }

    #[test]
    fn test_classify_point() {
        assert_eq!(classify_point((0, 0)), "at origin");
        assert_eq!(classify_point((0, 5)), "on y-axis");
        assert_eq!(classify_point((5, 0)), "on x-axis");
        assert_eq!(classify_point((3, 3)), "on y=x line");
        assert_eq!(classify_point((3, -3)), "on y=-x line");
        assert_eq!(classify_point((3, 5)), "in first quadrant");
        assert_eq!(classify_point((-3, 5)), "in second quadrant");
        assert_eq!(classify_point((-3, -5)), "in third quadrant");
        assert_eq!(classify_point((3, -5)), "in fourth quadrant");
    }

    #[test]
    fn test_print_coordinates() {
        let point = (10, 20);
        assert_eq!(print_coordinates(&point), "Coordinates: (10, 20)");
    }
}
