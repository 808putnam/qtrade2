//! Foreign Function Interface (FFI) Examples
//!
//! This module demonstrates working with C libraries and foreign code:
//! - Declaring external functions
//! - Passing data between Rust and C
//! - Memory safety in FFI
//! - Callbacks
//! - Wrapping C libraries in safe Rust APIs

// For this example, we'll show how to declare FFI functions
// but we won't actually call them since we don't have the C
// libraries available in this environment

use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};

/// Problem: Declare external C functions
///
/// Define FFI bindings to standard C library functions
#[allow(unused)]
extern "C" {
    // C standard library functions
    fn strlen(s: *const c_char) -> usize;
    fn printf(format: *const c_char, ...) -> c_int;
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);

    // POSIX functions
    fn open(path: *const c_char, flags: c_int, ...) -> c_int;
    fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
    fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
    fn close(fd: c_int) -> c_int;
}

/// Problem: Create a safe wrapper for C functions
///
/// Wrap a C string length function in a safe Rust API
#[allow(unused)]
pub fn safe_strlen(s: &str) -> Result<usize, std::str::Utf8Error> {
    // Convert Rust string to C string
    let c_string = CString::new(s).expect("CString::new failed");

    // Call the C function safely
    let length = unsafe { strlen(c_string.as_ptr()) };

    Ok(length)
}

/// Problem: Define a C struct for FFI
///
/// Create a struct that matches a C struct's memory layout
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Point {
    x: f64,
    y: f64,
}

// In C:
// struct Point {
//     double x;
//     double y;
// };

/// Problem: Define a function that could be called from C
///
/// Create a function that can be exported to C code
#[no_mangle]
pub extern "C" fn calculate_distance(p1: Point, p2: Point) -> f64 {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    (dx * dx + dy * dy).sqrt()
}

/// Problem: Define a callback function type
///
/// Create a type for function pointers that can be passed to C
pub type LogCallback = extern "C" fn(message: *const c_char);

/// Problem: Implement a function that takes a callback
///
/// Create a function that accepts and calls a C-compatible callback
#[no_mangle]
pub extern "C" fn perform_with_logging(
    value: c_int,
    callback: Option<LogCallback>,
) -> c_int {
    let result = value * 2;

    // Call the callback if provided
    if let Some(log_fn) = callback {
        let message = CString::new(format!("Calculated result: {}", result))
            .expect("CString::new failed");
        log_fn(message.as_ptr());
    }

    result
}

/// Problem: Create a safe wrapper around a C library
///
/// Implement a memory-safe file API around C's open/read/write/close
#[allow(unused)]
pub struct File {
    fd: c_int,
}

#[allow(unused)]
impl File {
    pub fn open(path: &str) -> Result<Self, String> {
        let c_path = match CString::new(path) {
            Ok(p) => p,
            Err(_) => return Err("Path contains null bytes".to_string()),
        };

        // Constants from C headers
        const O_RDONLY: c_int = 0;

        let fd = unsafe { open(c_path.as_ptr(), O_RDONLY) };

        if fd < 0 {
            Err("Could not open file".to_string())
        } else {
            Ok(File { fd })
        }
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize, String> {
        let read_count = unsafe {
            read(
                self.fd,
                buf.as_mut_ptr() as *mut c_void,
                buf.len(),
            )
        };

        if read_count < 0 {
            Err("Read error".to_string())
        } else {
            Ok(read_count as usize)
        }
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, String> {
        let write_count = unsafe {
            write(
                self.fd,
                buf.as_ptr() as *const c_void,
                buf.len(),
            )
        };

        if write_count < 0 {
            Err("Write error".to_string())
        } else {
            Ok(write_count as usize)
        }
    }
}

#[allow(unused)]
impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            close(self.fd);
        }
    }
}

/// Problem: Pass arrays between Rust and C
///
/// Create functions to handle array passing in FFI
#[no_mangle]
pub extern "C" fn sum_array(data: *const c_int, length: c_uint) -> c_int {
    // Safety: We trust the caller to provide a valid array with the specified length
    let slice = unsafe {
        if data.is_null() || length == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(data, length as usize)
        }
    };

    slice.iter().fold(0, |acc, &x| acc + x)
}

/// Problem: Handle C strings correctly
///
/// Process strings coming from C code
#[no_mangle]
pub extern "C" fn process_c_string(input: *const c_char) -> *mut c_char {
    // Handle null pointer
    if input.is_null() {
        let result = CString::new("Input was null").expect("CString::new failed");
        return result.into_raw();
    }

    // Convert C string to Rust string
    let c_str = unsafe { CStr::from_ptr(input) };
    let rust_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => "Input was not valid UTF-8",
    };

    // Process the string (here we just uppercase it)
    let processed = rust_str.to_uppercase();

    // Convert back to C string
    let result = CString::new(processed).expect("CString::new failed");
    result.into_raw() // Note: Caller is responsible for freeing this memory
}

/// Free a string created by process_c_string
#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            // Retake ownership and drop
            let _ = CString::from_raw(ptr);
        }
    }
}

/// Problem: Create a struct with methods callable from C
///
/// Define an opaque type for C to interact with
pub struct Counter {
    value: usize,
}

#[no_mangle]
pub extern "C" fn counter_create() -> *mut Counter {
    let counter = Box::new(Counter { value: 0 });
    Box::into_raw(counter)
}

#[no_mangle]
pub extern "C" fn counter_increment(counter: *mut Counter) -> usize {
    let counter = unsafe {
        if counter.is_null() {
            return 0;
        }
        &mut *counter
    };

    counter.value += 1;
    counter.value
}

#[no_mangle]
pub extern "C" fn counter_get(counter: *const Counter) -> usize {
    let counter = unsafe {
        if counter.is_null() {
            return 0;
        }
        &*counter
    };

    counter.value
}

#[no_mangle]
pub extern "C" fn counter_destroy(counter: *mut Counter) {
    if !counter.is_null() {
        unsafe {
            let _ = Box::from_raw(counter);
        }
    }
}

// Since we can't actually call these functions in this environment,
// we'll provide example C code showing how these would be used

/* Example C code:

// Example for using calculate_distance
#include <stdio.h>

struct Point {
    double x;
    double y;
};

// Import the Rust function
extern double calculate_distance(struct Point p1, struct Point p2);

int main() {
    struct Point p1 = {1.0, 2.0};
    struct Point p2 = {4.0, 6.0};

    double dist = calculate_distance(p1, p2);
    printf("Distance: %f\n", dist);

    return 0;
}

// Example for using perform_with_logging
#include <stdio.h>

// Define the callback type
typedef void (*LogCallback)(const char* message);

// Import the Rust function
extern int perform_with_logging(int value, LogCallback callback);

// Callback implementation
void log_message(const char* message) {
    printf("Log: %s\n", message);
}

int main() {
    int result = perform_with_logging(5, log_message);
    printf("Result: %d\n", result);
    return 0;
}

// Example for using Counter
#include <stdio.h>

// Opaque struct declaration
typedef struct Counter Counter;

// Import the Rust functions
extern Counter* counter_create(void);
extern size_t counter_increment(Counter* counter);
extern size_t counter_get(const Counter* counter);
extern void counter_destroy(Counter* counter);

int main() {
    Counter* counter = counter_create();

    printf("Initial value: %zu\n", counter_get(counter));

    counter_increment(counter);
    counter_increment(counter);

    printf("After increments: %zu\n", counter_get(counter));

    counter_destroy(counter);

    return 0;
}
*/
