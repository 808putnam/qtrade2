//! Unsafe Rust Examples
//!
//! This module demonstrates working with unsafe Rust:
//! - Raw pointers
//! - C FFI (Foreign Function Interface)
//! - Mutable statics
//! - Union types
//! - Implementing unsafe traits
//! - Dereferencing raw pointers
//! - Safe abstractions over unsafe code

use std::slice;

/// Problem: Implement a safe abstraction around unsafe code
///
/// Create a function that safely splits a slice into two mutable parts
pub fn split_at_mut<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    // Check if index is valid
    assert!(mid <= slice.len(), "mid index out of bounds");

    // SAFETY:
    // 1. We've verified that `mid` is within bounds
    // 2. The two slices are non-overlapping as they're split at `mid`
    // 3. We're still respecting Rust's borrowing rules with the resulting slices
    unsafe {
        let len = slice.len();
        let ptr = slice.as_mut_ptr();

        (
            slice::from_raw_parts_mut(ptr, mid),
            slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

/// Problem: Working with raw pointers
///
/// Create a function to swap two values using raw pointers
pub fn swap_raw<T>(a: &mut T, b: &mut T) {
    // SAFETY:
    // 1. We're converting references to raw pointers, which is always safe
    // 2. The pointers are valid because they come from valid references
    // 3. The pointers point to initialized values of type T
    // 4. We're not creating any aliasing issues since the dereferencing is brief
    //    and doesn't escape the unsafe block
    unsafe {
        let a_ptr: *mut T = a;
        let b_ptr: *mut T = b;

        // We need a temporary value to perform the swap
        // Create a MaybeUninit to avoid assuming T is Copy
        let mut temp = std::mem::MaybeUninit::uninit();

        // Copy a into temp
        std::ptr::copy_nonoverlapping(a_ptr, temp.as_mut_ptr(), 1);

        // Copy b into a
        std::ptr::copy_nonoverlapping(b_ptr, a_ptr, 1);

        // Copy temp into b
        std::ptr::copy_nonoverlapping(temp.as_ptr() as *const T, b_ptr, 1);
    }
}

/// Problem: Use mutable static variables
///
/// Create and work with a mutable static variable
static mut COUNTER: u32 = 0;

pub fn get_counter() -> u32 {
    // SAFETY:
    // This is unsafe because mutable statics can cause data races.
    // In a real application, you would use synchronization primitives.
    unsafe { COUNTER }
}

pub fn increment_counter() -> u32 {
    // SAFETY:
    // We're the only ones accessing this static in this example.
    // In real code, you would need synchronization.
    unsafe {
        COUNTER += 1;
        COUNTER
    }
}

/// Problem: Create an unsafe trait
///
/// Define a trait that requires unsafe implementation
pub unsafe trait UnsafeExample {
    fn dangerous_operation(&self);
}

pub struct MyType;

// SAFETY:
// The implementor must ensure that dangerous_operation is used safely.
// Here we're just printing, but in a real scenario, this could involve
// raw memory manipulation or other unsafe operations.
unsafe impl UnsafeExample for MyType {
    fn dangerous_operation(&self) {
        println!("This operation is marked as dangerous!");
    }
}

/// Problem: Use unions for memory-efficient representations
///
/// Create a union representing different interpretations of the same memory
#[repr(C)]
pub union IntOrFloat {
    pub i: i32,
    pub f: f32,
}

impl IntOrFloat {
    pub fn new_int(value: i32) -> Self {
        IntOrFloat { i: value }
    }

    pub fn new_float(value: f32) -> Self {
        IntOrFloat { f: value }
    }

    pub fn get_int(&self) -> i32 {
        // SAFETY:
        // We're assuming the union currently holds an integer.
        // The caller must ensure this is true.
        unsafe { self.i }
    }

    pub fn get_float(&self) -> f32 {
        // SAFETY:
        // We're assuming the union currently holds a float.
        // The caller must ensure this is true.
        unsafe { self.f }
    }
}

/// Problem: Implement a custom smart pointer with raw pointers
///
/// Create a simple reference-counted pointer using unsafe code
pub struct SimpleRc<T> {
    ptr: *mut RcInner<T>,
}

struct RcInner<T> {
    value: T,
    count: usize,
}

impl<T> SimpleRc<T> {
    pub fn new(value: T) -> Self {
        // Allocate on the heap
        let inner = Box::new(RcInner {
            value,
            count: 1,
        });

        // Convert the Box to a raw pointer and forget about it
        // so it won't be dropped automatically
        let ptr = Box::into_raw(inner);

        SimpleRc { ptr }
    }

    pub fn clone(&self) -> Self {
        // SAFETY:
        // The pointer is valid because we only create it in `new`,
        // and we never free it except in Drop.
        unsafe {
            (*self.ptr).count += 1;
        }

        SimpleRc { ptr: self.ptr }
    }

    pub fn get_count(&self) -> usize {
        // SAFETY:
        // The pointer is valid (see above).
        unsafe { (*self.ptr).count }
    }

    pub fn get_ref(&self) -> &T {
        // SAFETY:
        // The pointer is valid, and the reference doesn't outlive `self`.
        unsafe { &(*self.ptr).value }
    }
}

impl<T> Drop for SimpleRc<T> {
    fn drop(&mut self) {
        // SAFETY:
        // The pointer is valid (as described above).
        unsafe {
            (*self.ptr).count -= 1;

            if (*self.ptr).count == 0 {
                // If this is the last reference, reconstruct the Box and let it drop
                drop(Box::from_raw(self.ptr));
            }
        }
    }
}

// Implement Clone properly
impl<T> Clone for SimpleRc<T> {
    fn clone(&self) -> Self {
        Self::clone(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_at_mut() {
        let mut v = vec![1, 2, 3, 4, 5];

        let (left, right) = split_at_mut(&mut v, 3);

        assert_eq!(left, &mut [1, 2, 3]);
        assert_eq!(right, &mut [4, 5]);

        // Modify both slices
        left[0] = 10;
        right[0] = 40;

        assert_eq!(v, vec![10, 2, 3, 40, 5]);
    }

    #[test]
    #[should_panic(expected = "mid index out of bounds")]
    fn test_split_at_mut_out_of_bounds() {
        let mut v = vec![1, 2, 3];
        let _ = split_at_mut(&mut v, 4); // This should panic
    }

    #[test]
    fn test_swap_raw() {
        let mut a = 5;
        let mut b = 10;

        swap_raw(&mut a, &mut b);

        assert_eq!(a, 10);
        assert_eq!(b, 5);

        // Test with a more complex type
        let mut s1 = String::from("hello");
        let mut s2 = String::from("world");

        swap_raw(&mut s1, &mut s2);

        assert_eq!(s1, "world");
        assert_eq!(s2, "hello");
    }

    #[test]
    fn test_counter() {
        // Reset counter before test
        unsafe { COUNTER = 0; }

        assert_eq!(get_counter(), 0);
        assert_eq!(increment_counter(), 1);
        assert_eq!(increment_counter(), 2);
        assert_eq!(get_counter(), 2);
    }

    #[test]
    fn test_int_or_float() {
        let mut value = IntOrFloat::new_int(42);
        assert_eq!(value.get_int(), 42);

        // Change to float
        value = IntOrFloat::new_float(3.14);
        let f = value.get_float();
        // Use approximate equality for floating point
        assert!((f - 3.14).abs() < 0.00001);
    }

    #[test]
    fn test_simple_rc() {
        let rc1 = SimpleRc::new(42);
        assert_eq!(rc1.get_count(), 1);

        {
            let rc2 = rc1.clone();
            assert_eq!(rc1.get_count(), 2);
            assert_eq!(rc2.get_count(), 2);

            assert_eq!(*rc1.get_ref(), 42);
            assert_eq!(*rc2.get_ref(), 42);
        }

        // rc2 is dropped here
        assert_eq!(rc1.get_count(), 1);
    }
}
