//! Implementation of [`Singly`].

use super::Collection;
use super::Linear;
use super::List;

use std::alloc;

/// Independently allocated elements connected via a single link.
///
/// Each element exists with a 'node', each of which are a separate allocated
/// object. These nodes are logically arranged in [`Self::Linear`] fashion
/// where each element links to the element after it and nothing else.
///
/// See also: [Wikipedia](https://en.wikipedia.org/wiki/Linked_list).
pub struct Singly<T> {
    /// The contained element for this node.
    element: T,

    /// The next element, if there is one.
    next: Option<Box<Singly<T>>>,
}

impl<'a, T: 'a> Collection<'a> for Singly<T> {
    type Element = T;

    fn count(&self) -> usize {
        let mut current = self;
        let mut count: usize = 1;

        while let Some(next) = &current.next {
            current = next;

            if let Some(incremented) = count.checked_add(1) {
                count = incremented;
            } else {
                panic!("too many elements to count");
            }
        }

        count
    }
}
