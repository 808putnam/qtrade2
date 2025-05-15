//! Smart Pointers and Memory Management Examples
//!
//! This module demonstrates various smart pointers and memory management patterns:
//! - Box<T> for heap allocation
//! - Rc<T> for shared ownership
//! - Arc<T> for thread-safe shared ownership
//! - RefCell<T> and Cell<T> for interior mutability
//! - Combine patterns like Rc<RefCell<T>>
//! - Custom smart pointers

use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Problem: Implement a custom smart pointer
///
/// Create a smart pointer that wraps a value and provides deref functionality
pub struct MyBox<T>(T);

impl<T> MyBox<T> {
    pub fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Problem: Create a recursive data structure
///
/// Demonstrate how to create a recursive data structure using Box
#[derive(Debug)]
pub enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

impl<T> List<T> {
    pub fn new() -> List<T> {
        List::Nil
    }

    pub fn prepend(self, elem: T) -> List<T> {
        List::Cons(elem, Box::new(self))
    }

    pub fn len(&self) -> usize {
        match self {
            List::Cons(_, tail) => 1 + tail.len(),
            List::Nil => 0,
        }
    }
}

/// Problem: Implement reference counting for shared data
///
/// Show how to share ownership of data among multiple owners
pub fn share_data() -> (Rc<String>, Rc<String>) {
    let data = Rc::new(String::from("shared data"));

    let data1 = Rc::clone(&data);
    let data2 = Rc::clone(&data);

    (data1, data2)
}

/// Problem: Create a mutable shared value with interior mutability
///
/// Use RefCell to allow mutable access to shared data
pub fn interior_mutability() -> Rc<RefCell<Vec<i32>>> {
    let data = Rc::new(RefCell::new(vec![1, 2, 3]));

    let data_ref = Rc::clone(&data);
    // Mutably borrow and modify
    data_ref.borrow_mut().push(4);

    data
}

/// Problem: Thread-safe shared ownership
///
/// Use Arc and Mutex for thread-safe shared ownership
pub fn thread_safe_sharing() -> Arc<Mutex<Vec<String>>> {
    let data = Arc::new(Mutex::new(vec![
        "shared".to_string(),
        "among".to_string(),
        "threads".to_string()
    ]));

    let data_clone = Arc::clone(&data);

    std::thread::spawn(move || {
        let mut vec = data_clone.lock().unwrap();
        vec.push("safely".to_string());
    }).join().unwrap();

    data
}

/// Problem: Create a data structure with internal mutability without RefCell
///
/// Use Cell for simple copy types that need internal mutability
pub struct Counter {
    count: Cell<u32>,
}

impl Counter {
    pub fn new() -> Self {
        Counter { count: Cell::new(0) }
    }

    pub fn increment(&self) {
        let current = self.count.get();
        self.count.set(current + 1);
    }

    pub fn get(&self) -> u32 {
        self.count.get()
    }
}

/// Problem: Implement the dropped pattern
///
/// Create a type that runs code when it's dropped
pub struct ResourceGuard<'a> {
    name: &'a str,
}

impl<'a> ResourceGuard<'a> {
    pub fn new(name: &'a str) -> Self {
        println!("Resource '{}' acquired", name);
        ResourceGuard { name }
    }
}

impl<'a> Drop for ResourceGuard<'a> {
    fn drop(&mut self) {
        println!("Resource '{}' released", self.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_box() {
        let x = 5;
        let y = MyBox::new(x);
        assert_eq!(5, *y);
    }

    #[test]
    fn test_list() {
        let list = List::new().prepend(1).prepend(2).prepend(3);
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_shared_data() {
        let (a, b) = share_data();
        assert_eq!(*a, "shared data");
        assert_eq!(*a, *b);
    }

    #[test]
    fn test_interior_mutability() {
        let data = interior_mutability();
        assert_eq!(*data.borrow(), vec![1, 2, 3, 4]);

        // We can still modify it
        data.borrow_mut().push(5);
        assert_eq!(*data.borrow(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_counter() {
        let counter = Counter::new();
        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 2);
    }
}
