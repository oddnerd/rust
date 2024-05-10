//! Implementation of [`Singly`].

use super::Collection;
use super::Linear;
use super::List;

// TODO(oddnerd): examples for everything

/// Independently allocated elements connected via a single link.
///
/// Each element exists within separate allocated object, referred to as a
/// node. Each node contains a single pointer which is said to 'link' to
/// subsequent elements in a [`super::Linear`] sequence. Therefore, [`Self`]
/// points to the first node (if any) and each subsequent node points to the
/// next until the last element which points to nothing as visualized below:
///
/// ```text
///         +-------+    +---------+    +---------+           +------+
/// Self -> | first | -> | element | -> | element | -> ... -> | last |
///         +-------+    +---------+    +---------+           +------+
/// ```
///
/// See also: [Wikipedia](https://en.wikipedia.org/wiki/Linked_list)
pub struct Singly<T> {
    /// The contained elements.
    elements: Option<Box<Node<T>>>,
}

/// An independently allocated element contained within some [`Singly`].
struct Node<T> {
    /// The underlying contained value.
    element: T,

    /// Link to the rest of the list.
    next: Option<Box<Node<T>>>,
}

impl<T> Default for Singly<T> {
    /// Create an empty instance of [`Singly`].
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory for the result.
    fn default() -> Self {
        Singly { elements: None }
    }
}

impl<T> Drop for Singly<T> {
    /// Iteratively drop all contained elements.
    ///
    /// The default destructor implementation will _NOT_ be tail recursive,
    /// hence this provided iterative method to prevent stack overflow.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    fn drop(&mut self) {
        let mut current = self.elements.take();

        while let Some(mut node) = current {
            current = node.next.take();
        }
    }
}

impl<T> Singly<T> {
    pub fn prepend(&mut self, value: T) {
        let new = Box::new(Node {
            element: value,
            next: self.elements.take(),
        });

        self.elements = Some(new);
    }

    pub fn remove_front(&mut self) -> Option<T> {
        self.elements.take().map(|node| {
            self.elements = node.next;

            node.element
        })
    }

    pub fn peek_front(&self) -> Option<&T> {
        self.elements.as_ref().map(|node| &node.element)
    }

    pub fn peek_front_mut(&mut self) -> Option<&mut T> {
        self.elements.as_mut().map(|node| &mut node.element)
    }
}

impl<T> core::ops::Index<usize> for Singly<T> {
    type Output = T;

    /// Obtain an immutable reference to the element at `index`.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    fn index(&self, index: usize) -> &Self::Output {
        let Some(mut current) = self.elements.as_ref() else {
            panic!("no elements contained");
        };

        for _ in 0..index {
            if let Some(next) = current.next.as_ref() {
                current = next;
            } else {
                panic!("index out of bounds");
            }
        }

        &current.element
    }
}

impl<T> core::ops::IndexMut<usize> for Singly<T> {
    /// Obtain a mutable reference to the element at `index`.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let Some(mut current) = self.elements.as_mut() else {
            panic!("no elements contained");
        };

        for _ in 0..index {
            if let Some(next) = current.next.as_mut() {
                current = next;
            } else {
                panic!("index out of bounds");
            }
        }

        &mut current.element
    }
}

impl<T> Iterator for Singly<T> {
    type Item = T;

    /// Obtain the first element.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    fn next(&mut self) -> Option<Self::Item> {
        todo!("pop the front element");
    }

    /// Query how many elements are contained.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.count();
        (count, Some(count))
    }
}

impl<T> DoubleEndedIterator for Singly<T> {
    /// Obtain the last element.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    fn next_back(&mut self) -> Option<Self::Item> {
        todo!("pop the last element");
    }
}

impl<T> ExactSizeIterator for Singly<T> {}

impl<T> core::iter::FusedIterator for Singly<T> {}

impl<T> Extend<T> for Singly<T> {
    /// Append each element.
    ///
    /// # Panics
    /// This method panics if memory cannot be allocated.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(N) memory.
    fn extend<Iter: IntoIterator<Item = T>>(&mut self, iter: Iter) {
        todo!("append each element");
    }
}

impl<T> FromIterator<T> for Singly<T> {
    /// Construct an instance with elements from an iterator.
    ///
    /// # Panics
    /// This method panics if memory cannot be allocated.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(N) memory for the result.
    fn from_iter<Iter: IntoIterator<Item = T>>(iter: Iter) -> Self {
        let mut instance = Singly::<T>::default();
        instance.extend(iter);
        instance
    }
}

impl<'a, T: 'a> Collection for Singly<T> {
    type Element = T;

    /// Query how many elements are contained.
    ///
    /// # Panics
    /// Panics if the number of elements contained is more than [`usize::MAX`].
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    fn count(&self) -> usize {
        let Some(mut current) = self.elements.as_ref() else {
            return 0;
        };

        let mut count: usize = 1;

        while let Some(next) = current.next.as_ref() {
            if let Some(incremented) = count.checked_add(1) {
                count = incremented;
            } else {
                panic!("too many elements for size type");
            }

            current = next;
        }

        count
    }
}

impl<T> Linear for Singly<T> {
    /// Iterate over the elements by immutable reference.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = &Self::Element> + ExactSizeIterator + core::iter::FusedIterator
    {
        Iter {
            next: self.elements.as_deref(),
            previous_back: None,
        }
    }

    /// Iterate over the elements by mutable reference.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    fn iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = &mut Self::Element>
           + ExactSizeIterator
           + core::iter::FusedIterator {
        IterMut {
            next: self.elements.as_deref_mut(),
            previous_back: None,
        }
    }
}

impl<T> List for Singly<T> {
    /// Move an `element` into a new node at `index`.
    ///
    /// # Panics
    /// The Rust runtime might abort if allocation fails, panics otherwise.
    ///
    /// # Performance
    /// This method takes O(N) times and consumes O(1) memory.
    fn insert(
        &mut self,
        index: usize,
        element: Self::Element,
    ) -> Result<&mut Self::Element, Self::Element> {
        let new = Box::new(Node {
            element,
            next: None,
        });

        if self.elements.is_none() && index == 0 {
            let new = self.elements.insert(new);
            return Ok(&mut new.element);
        }

        let mut current = self.elements.as_deref_mut();

        for _ in 0..index {
            current = current.and_then(|current| current.next.as_deref_mut());
        }

        if let Some(preceding) = current {
            let succeeding = preceding.next.take();
            let new = preceding.next.insert(new);
            new.next = succeeding;

            Ok(&mut new.element)
        } else {
            Err(new.element)
        }
    }

    /// Move the element at `index` out, if it exists.
    ///
    /// # Performance
    /// This method takes O(N) times and consumes O(1) memory.
    fn remove(&mut self, index: usize) -> Option<Self::Element> {
        if index == 0 {
            let node = self.elements.take()?;
            self.elements = node.next;
            return Some(node.element);
        }

        let mut current = self.elements.as_deref_mut();

        for _ in 0..index - 1 {
            current = current.and_then(|current| current.next.as_deref_mut());
        }

        if let Some(preceding) = current {
            let mut node = preceding.next.take()?;
            let succeeding = node.next.take();
            preceding.next = succeeding;

            Some(node.element)
        } else {
            unreachable!("at least two elements");
        }
    }

    /// Efficiently remove the elements within the given index `range`.
    ///
    /// Using [`Self::remove`] would be inefficient because each removal would
    /// require traversing the list to the given index which is O(N^2) time,
    /// whereas this method traverses the list only once there being O(N).
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    fn drain(
        &mut self,
        range: impl core::ops::RangeBounds<usize>,
    ) -> impl DoubleEndedIterator<Item = Self::Element> + ExactSizeIterator {
        let start = match range.start_bound() {
            core::ops::Bound::Included(start) => *start,
            core::ops::Bound::Excluded(start) => match start.checked_add(1) {
                Some(start) => start,
                None => {
                    // TODO: does this special case drop?
                    return Drain {
                        underlying: Some(self),
                        preceding: None,
                        next: None,
                        remaining: 0,
                    };
                }
            },
            core::ops::Bound::Unbounded => 0,
        };

        let remaining = match range.end_bound() {
            core::ops::Bound::Included(end) => (end - start).saturating_add(1),
            core::ops::Bound::Excluded(end) => end - start,
            core::ops::Bound::Unbounded => self.len() - start,
        };

        if start == 0 {
            let next = self.elements.take();

            return Drain {
                underlying: Some(self),
                preceding: None,
                next,
                remaining,
            };
        }

        let mut current = self.elements.as_deref_mut();

        for _ in 0..start {
            current = current.and_then(|current| current.next.as_deref_mut());
        }

        if let Some(preceding) = current {
            let next = preceding.next.take();

            Drain {
                underlying: None,
                preceding: Some(preceding),
                next,
                remaining,
            }
        } else {
            unreachable!("preceding elements");
        }
    }

    fn withdraw(
        &mut self,
        predicate: impl FnMut(&Self::Element) -> bool,
    ) -> impl DoubleEndedIterator<Item = Self::Element> {
        todo!("create custom iterator");
    }
}

/// Immutable iterator over a [`Singly`].
struct Iter<'a, T> {
    /// The next element to yield, if any.
    next: Option<&'a Node<T>>,

    /// The previously yielded element from the back, if any.
    previous_back: Option<&'a Node<T>>,
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    /// Obtain the next element from the front, if any.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|current| {
            self.next = current.next.as_deref();

            if let (Some(next), Some(sentinel)) = (self.next, self.previous_back) {
                if core::ptr::addr_eq(next, sentinel) {
                    self.next = None;
                }
            }

            &current.element
        })
    }

    /// Query how many elements have yet to be yielded.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Some(mut current) = self.next else {
            return (0, Some(0));
        };

        let mut count: usize = 1;

        while let Some(next) = current.next.as_deref() {
            if let Some(incremented) = count.checked_add(1) {
                count = incremented;
            } else {
                unreachable!("more than `usize::MAX` elements");
            }

            if let Some(sentinel) = self.previous_back {
                if core::ptr::addr_eq(next, sentinel) {
                    break;
                }
            }

            current = next;
        }

        (count, Some(count))
    }
}

impl<'a, T: 'a> DoubleEndedIterator for Iter<'a, T> {
    /// Obtain the next element from the back, if any.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut current = self.next?;

        while let Some(next) = current.next.as_deref() {
            if let Some(sentinel) = self.previous_back {
                if core::ptr::addr_eq(next, sentinel) {
                    break;
                }
            }

            current = next;
        }

        self.previous_back = Some(current);

        if let Some(next) = self.next {
            if core::ptr::addr_eq(next, current) {
                self.next = None;
            }
        }

        Some(&current.element)
    }
}

impl<'a, T: 'a> ExactSizeIterator for Iter<'a, T> {}

impl<'a, T: 'a> core::iter::FusedIterator for Iter<'a, T> {}

/// Mutable iterator over a [`Singly`].
struct IterMut<'a, T> {
    /// The next element to yield, if any.
    next: Option<&'a mut Node<T>>,

    /// The previously yielded element from the back, if any.
    previous_back: Option<*const Node<T>>,
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    /// Obtain the next element from the front, if any.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|current| {
            self.next = current.next.as_deref_mut();

            if let (Some(next), Some(sentinel)) = (self.next.as_deref(), self.previous_back) {
                if core::ptr::addr_eq(next, sentinel) {
                    self.next = None;
                }
            }

            &mut current.element
        })
    }

    /// Query how many elements have yet to be yielded.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Some(mut current) = self.next.as_deref() else {
            return (0, Some(0));
        };

        let mut count: usize = 1;

        while let Some(next) = current.next.as_deref() {
            if let Some(incremented) = count.checked_add(1) {
                count = incremented;
            } else {
                unreachable!("more than `usize::MAX` elements");
            }

            if let Some(sentinel) = self.previous_back {
                if core::ptr::addr_eq(next, sentinel) {
                    break;
                }
            }

            current = next;
        }

        (count, Some(count))
    }
}

impl<'a, T: 'a> DoubleEndedIterator for IterMut<'a, T> {
    /// Obtain the next element from the back, if any.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    fn next_back(&mut self) -> Option<Self::Item> {
        // TODO(oddnerd): this whole method is using pointers to work around reference
        // lifetime restrictions, therefore the validity of yielded references
        // ought to be questioned. Unit testing will hopefully validate the
        // quality of my assumptions?

        let mut current = core::ptr::from_mut(self.next.as_deref_mut()?);

        // SAFETY: the pointer non-null and aligned to an initialized object.
        while let Some(next) = unsafe { &mut *current }.next.as_deref_mut() {
            if let Some(sentinel) = self.previous_back {
                if core::ptr::addr_eq(next, sentinel) {
                    break;
                }
            }

            current = next;
        }

        self.previous_back = Some(current);

        if let Some(next) = self.next.as_deref_mut() {
            if core::ptr::addr_eq(next, current) {
                self.next = None;
            }
        }

        // SAFETY:
        // * we have a unique mutable reference to all elements of `Self`,
        // * this will _NEVER_ yield multiple references to the same element,
        //   (this includes preventing `next` (front) from referencing it)
        // * the yielded references has lifetime of `Self`.
        Some(&mut unsafe { &mut *current }.element)
    }
}

impl<'a, T: 'a> ExactSizeIterator for IterMut<'a, T> {}

impl<'a, T: 'a> core::iter::FusedIterator for IterMut<'a, T> {}

/// By-value iterator over a range of indices.
struct Drain<'a, T> {
    /// The underlying elements being drained from.
    underlying: Option<&'a mut Singly<T>>,

    /// The node preceding those being drained, if any.
    preceding: Option<&'a mut Node<T>>,

    /// The remaining elements of the list, if any.
    next: Option<Box<Node<T>>>,

    /// The number of elements yet to be yielded.
    remaining: usize,
}

impl<'a, T: 'a> Drop for Drain<'a, T> {
    /// Remove elements yet to be drained and repair the underlying [`Singly`].
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    fn drop(&mut self) {
        self.for_each(drop);

        if let Some(preceding) = self.preceding.take() {
            preceding.next = self.next.take();
        } else {
            let Some(underlying) = self.underlying.take() else {
                unreachable!("constructor logic error");
            };

            underlying.elements = self.next.take();
        }
    }
}

impl<'a, T: 'a> Iterator for Drain<'a, T> {
    type Item = T;

    /// Obtain the next element from the front, if any exists.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    fn next(&mut self) -> Option<Self::Item> {
        let mut current = self.next.take()?;

        self.next = current.next.take();

        self.remaining -= 1;

        Some(current.element)
    }

    /// Query how many elements have yet to be yielded.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut current = &self.next;

        for count in 0..self.remaining {
            let Some(node) = current else {
                return (count, Some(count));
            };

            current = &node.next;
        }

        (self.remaining, Some(self.remaining))
    }
}

impl<'a, T: 'a> DoubleEndedIterator for Drain<'a, T> {
    /// Obtain the next element from the back, if any exists.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut current = self.next.as_deref_mut();

        for _ in 0..self.remaining - 1 {
            current = current.and_then(|current| current.next.as_deref_mut());
        }

        if let Some(preceding) = current {
            let mut node = preceding.next.take()?;
            let succeeding = node.next.take();
            preceding.next = succeeding;

            self.remaining -= 1;

            Some(node.element)
        } else {
            None
        }
    }
}

impl<'a, T: 'a> ExactSizeIterator for Drain<'a, T> {}

impl<'a, T: 'a> core::iter::FusedIterator for Drain<'a, T> {}
