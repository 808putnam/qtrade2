//! Iterators and Functional Programming Examples
//!
//! This module demonstrates:
//! - Creating custom iterators
//! - Using map, filter, fold and other functional methods
//! - Closures and higher-order functions
//! - Iterator combinators and adapters
//! - Lazy evaluation with iterators
//! - Iterator for custom data structures

use std::iter::FromIterator;

/// Problem: Implement a custom iterator
///
/// Create a Counter iterator that generates a sequence of numbers
#[derive(Debug)]
pub struct Counter {
    count: usize,
    max: usize,
}

impl Counter {
    pub fn new(max: usize) -> Self {
        Counter { count: 0, max }
    }
}

impl Iterator for Counter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.max {
            let current = self.count;
            self.count += 1;
            Some(current)
        } else {
            None
        }
    }
}

/// Problem: Create a custom collection with an iterator
///
/// Implement a simple binary tree with an in-order iterator
#[derive(Debug, PartialEq)]
pub enum BinaryTree<T> {
    Empty,
    Node(T, Box<BinaryTree<T>>, Box<BinaryTree<T>>),
}

impl<T: Ord> BinaryTree<T> {
    pub fn new() -> Self {
        BinaryTree::Empty
    }

    pub fn insert(&mut self, value: T) {
        match self {
            BinaryTree::Empty => {
                *self = BinaryTree::Node(value, Box::new(BinaryTree::Empty), Box::new(BinaryTree::Empty));
            }
            BinaryTree::Node(ref mut v, ref mut left, ref mut right) => {
                if value < *v {
                    left.insert(value);
                } else {
                    right.insert(value);
                }
            }
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        match self {
            BinaryTree::Empty => false,
            BinaryTree::Node(v, left, right) => {
                if value == v {
                    true
                } else if value < v {
                    left.contains(value)
                } else {
                    right.contains(value)
                }
            }
        }
    }

    pub fn inorder_iter(&self) -> InOrderIterator<T> {
        let mut stack = Vec::new();
        let mut current = self;
        InOrderIterator { stack, current }
    }
}

/// In-order iterator for BinaryTree
pub struct InOrderIterator<'a, T: 'a> {
    stack: Vec<&'a BinaryTree<T>>,
    current: &'a BinaryTree<T>,
}

impl<'a, T> Iterator for InOrderIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // Go left as far as possible
        while let BinaryTree::Node(_, left, _) = self.current {
            self.stack.push(self.current);
            self.current = left;
        }

        // Pop from stack and go right once
        if let Some(node) = self.stack.pop() {
            if let BinaryTree::Node(value, _, right) = node {
                self.current = right;
                return Some(value);
            }
        }

        None
    }
}

/// Problem: Implement iterator combinators from scratch
///
/// Create your own versions of common iterator combinators
pub trait MyIteratorExt: Iterator {
    fn my_map<B, F>(self, f: F) -> MyMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> B,
    {
        MyMap { iter: self, f }
    }

    fn my_filter<P>(self, predicate: P) -> MyFilter<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        MyFilter {
            iter: self,
            predicate,
        }
    }

    fn my_fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let mut acc = init;
        while let Some(x) = self.next() {
            acc = f(acc, x);
        }
        acc
    }
}

// Implement MyIteratorExt for all iterators
impl<T: ?Sized> MyIteratorExt for T where T: Iterator {}

pub struct MyMap<I, F> {
    iter: I,
    f: F,
}

impl<B, I: Iterator, F> Iterator for MyMap<I, F>
where
    F: FnMut(I::Item) -> B,
{
    type Item = B;

    fn next(&mut self) -> Option<B> {
        self.iter.next().map(&mut self.f)
    }
}

pub struct MyFilter<I, P> {
    iter: I,
    predicate: P,
}

impl<I: Iterator, P> Iterator for MyFilter<I, P>
where
    P: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        while let Some(x) = self.iter.next() {
            if (self.predicate)(&x) {
                return Some(x);
            }
        }
        None
    }
}

/// Problem: Use higher-order functions
///
/// Create a function that takes another function as an argument
pub fn apply_twice<F, T>(mut f: F, initial: T) -> T
where
    F: FnMut(T) -> T,
{
    let once = f(initial);
    f(once)
}

/// Problem: Implement FromIterator
///
/// Create a type that can be constructed from an iterator
#[derive(Debug, PartialEq)]
pub struct SumStats {
    count: usize,
    sum: i32,
    min: Option<i32>,
    max: Option<i32>,
}

impl FromIterator<i32> for SumStats {
    fn from_iter<T: IntoIterator<Item = i32>>(iter: T) -> Self {
        let mut stats = SumStats {
            count: 0,
            sum: 0,
            min: None,
            max: None,
        };

        for value in iter {
            stats.count += 1;
            stats.sum += value;
            stats.min = match stats.min {
                None => Some(value),
                Some(min) => Some(std::cmp::min(min, value)),
            };
            stats.max = match stats.max {
                None => Some(value),
                Some(max) => Some(std::cmp::max(max, value)),
            };
        }

        stats
    }
}

/// Problem: Demonstrate chaining iterators
///
/// Show the power of iterator composition
pub fn chain_iterators(v: &[i32]) -> Vec<i32> {
    v.iter()
        .filter(|x| **x % 2 == 0) // Take even numbers
        .map(|x| *x * *x)         // Square them
        .filter(|x| *x > 10)      // Take only those greater than 10
        .collect()                // Collect into a Vec
}

/// Problem: Implement a zip_with function
///
/// Combine two iterators with a function
pub fn zip_with<I, J, F, R>(
    iter1: I,
    iter2: J,
    mut zipper: F,
) -> impl Iterator<Item = R>
where
    I: IntoIterator,
    J: IntoIterator,
    F: FnMut(I::Item, J::Item) -> R,
{
    iter1
        .into_iter()
        .zip(iter2)
        .map(move |(a, b)| zipper(a, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let counter = Counter::new(3);
        let items: Vec<_> = counter.collect();
        assert_eq!(items, vec![0, 1, 2]);
    }

    #[test]
    fn test_binary_tree() {
        let mut tree = BinaryTree::new();
        tree.insert(5);
        tree.insert(3);
        tree.insert(7);
        tree.insert(2);
        tree.insert(4);

        assert!(tree.contains(&5));
        assert!(!tree.contains(&6));

        let values: Vec<_> = tree.inorder_iter().copied().collect();
        assert_eq!(values, vec![2, 3, 4, 5, 7]);
    }

    #[test]
    fn test_my_map() {
        let v = vec![1, 2, 3];
        let mapped: Vec<_> = v.iter().my_map(|x| x * 2).collect();
        assert_eq!(mapped, vec![2, 4, 6]);
    }

    #[test]
    fn test_my_filter() {
        let v = vec![1, 2, 3, 4, 5];
        let filtered: Vec<_> = v.iter().my_filter(|x| **x % 2 == 0).copied().collect();
        assert_eq!(filtered, vec![2, 4]);
    }

    #[test]
    fn test_my_fold() {
        let v = vec![1, 2, 3, 4, 5];
        let sum = v.iter().my_fold(0, |acc, x| acc + *x);
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_apply_twice() {
        let result = apply_twice(|x| x * 2, 3);
        assert_eq!(result, 12); // (3 * 2) * 2 = 12
    }

    #[test]
    fn test_sum_stats() {
        let stats: SumStats = vec![5, 1, 9, 2, 3].into_iter().collect();
        assert_eq!(
            stats,
            SumStats {
                count: 5,
                sum: 20,
                min: Some(1),
                max: Some(9),
            }
        );
    }

    #[test]
    fn test_chain_iterators() {
        let result = chain_iterators(&[1, 2, 3, 4, 5, 6]);
        assert_eq!(result, vec![16, 36]); // 4^2 and 6^2
    }

    #[test]
    fn test_zip_with() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];

        let result: Vec<_> = zip_with(a, b, |x, y| x + y).collect();
        assert_eq!(result, vec![5, 7, 9]);
    }
}
